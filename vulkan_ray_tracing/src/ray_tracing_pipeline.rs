use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::rc::Rc;

use ash::vk;
use nalgebra_glm as glm;
use vulkan_bootstrap::buffer::{Buffer, BufferBuilder, BufferType};
use vulkan_bootstrap::errors::VulkanError;
use vulkan_bootstrap::shader_module::ShaderModuleBuilder;
use vulkan_bootstrap::vulkan_context::VulkanContext;

use crate::acceleration_structure::{
    AccelerationStructure, AccelerationStructureBuilder, Instance,
};
use crate::bottom_level_acceleration_structure::{
    BottomLevelAccelerationStructure, BottomLevelAccelerationStructureBuilder,
};
use crate::descriptor_set::{DescriptorSet, DescriptorSetBuilder};
use crate::geometry_instance::{GeometryInstance, UniformBufferObject, Vertex};
use crate::pipeline::{Pipeline, PipelineBuilder};
use crate::ray_tracing::{RayTracing, RayTracingBuilder};
use crate::shader_binding_table::{ShaderBindingTable, ShaderBindingTableBuilder};

pub struct RayTracingPipeline {
    sbt: ShaderBindingTable,
    pipeline: Pipeline,
    descriptor_set: DescriptorSet,
    top_level_as: AccelerationStructure,
    _bottom_level_as: Vec<AccelerationStructure>,
    geometry_instance: GeometryInstance,
    camera_buffer: Buffer,
    ray_tracing: Rc<RayTracing>,
}

impl RayTracingPipeline {
    pub fn update_camera_buffer(&self, width: f32, height: f32) -> Result<(), VulkanError> {
        let model = glm::identity();
        let model_it = glm::inverse_transpose(model);
        let view = glm::look_at(
            &glm::vec3(4.0, 4.0, 4.0),
            &glm::vec3(0.0, 0.0, 0.0),
            &glm::vec3(0.0, 1.0, 0.0),
        );
        let aspect_ratio = width / height;
        let mut proj = glm::perspective(f32::to_radians(65.0), aspect_ratio, 0.1, 1000.0);
        proj[(1, 1)] = -proj[(1, 1)];
        let view_inverse = glm::inverse(&view);
        let proj_inverse = glm::inverse(&proj);

        let ubo = UniformBufferObject {
            model,
            view,
            proj,
            model_it,
            view_inverse,
            proj_inverse,
        };

        let data = &ubo as *const UniformBufferObject as *const c_void;

        self.camera_buffer.copy_data(data)
    }

    pub fn draw(&mut self, context: &mut VulkanContext) -> Result<(), VulkanError> {
        context.frame_begin()?;

        self.create_image_barrier(
            context,
            vk::AccessFlags::empty(),
            vk::AccessFlags::TRANSFER_WRITE,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::GENERAL,
        )?;

        self.descriptor_set.update_render_target(
            self.top_level_as.get(),
            context.get_current_back_buffer_view(),
            self.camera_buffer.get(),
            &self.geometry_instance,
        );

        let command_buffer = context.get_current_command_buffer();
        context.begin_render_pass();
        context.get_device().cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::RAY_TRACING_NV,
            self.pipeline.get(),
        );

        context.get_device().cmd_bind_descriptor_sets(
            command_buffer,
            self.pipeline.get_layout(),
            vk::PipelineBindPoint::RAY_TRACING_NV,
            &[self.descriptor_set.get()],
        );

        self.ray_tracing.cmd_trace_rays(
            command_buffer,
            self.sbt.get(),
            self.sbt.ray_gen_offset,
            self.sbt.get(),
            self.sbt.miss_offset,
            self.sbt.miss_entry_size,
            self.sbt.get(),
            self.sbt.hit_group_offset,
            self.sbt.hit_group_entry_size,
            context.get_swapchain().get_extent().width,
            context.get_swapchain().get_extent().height,
            1,
        );

        self.create_image_barrier(
            context,
            vk::AccessFlags::TRANSFER_WRITE,
            vk::AccessFlags::empty(),
            vk::ImageLayout::GENERAL,
            vk::ImageLayout::PRESENT_SRC_KHR,
        )?;

        context.get_device().cmd_next_subpass(command_buffer);
        context.end_render_pass();
        context.frame_end()?;
        context.frame_present()
    }

    fn create_image_barrier(
        &self,
        context: &VulkanContext,
        src_access_mask: vk::AccessFlags,
        dst_access_mask: vk::AccessFlags,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) -> Result<(), VulkanError> {
        let command_buffer = context.begin_single_time_commands()?;
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1)
            .build();

        let image_memory_barrier = vk::ImageMemoryBarrier::builder()
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask)
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(context.get_current_back_buffer())
            .subresource_range(subresource_range)
            .build();

        context.get_device().cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[image_memory_barrier],
        );
        context.end_single_time_commands(command_buffer)
    }
}

pub struct RayTracingPipelineBuilder<'a> {
    context: &'a VulkanContext,
    geometry_instance: Option<GeometryInstance>,
}

impl<'a> RayTracingPipelineBuilder<'a> {
    pub fn new(context: &'a VulkanContext) -> Self {
        RayTracingPipelineBuilder {
            context,
            geometry_instance: None,
        }
    }

    pub fn with_geometry_instance(mut self, geometry_instance: GeometryInstance) -> Self {
        self.geometry_instance = Some(geometry_instance);
        self
    }

    pub fn build(self) -> Result<RayTracingPipeline, VulkanError> {
        let ray_tracing = Rc::new(RayTracingBuilder::new(&self.context).build()?);

        let camera_buffer = BufferBuilder::new(&self.context)
            .with_type(BufferType::Uniform)
            .with_size(mem::size_of::<UniformBufferObject>() as u64)
            .build()?;

        let geometry_instance = self.geometry_instance.as_ref().unwrap();

        let (bottom_level_as, top_level_as) =
            self.create_acceleration_structures(Rc::clone(&ray_tracing), &geometry_instance)?;

        let descriptor_set = self.create_descriptor_set(&geometry_instance)?;

        let pipeline = self.create_pipeline(&ray_tracing, &descriptor_set)?;

        let sbt = self.create_shader_binding_table(&ray_tracing, &pipeline)?;

        Ok(RayTracingPipeline {
            ray_tracing,
            camera_buffer,
            geometry_instance: self.geometry_instance.unwrap(),
            _bottom_level_as: bottom_level_as,
            top_level_as,
            descriptor_set,
            pipeline,
            sbt,
        })
    }

    fn create_acceleration_structures(
        &self,
        ray_tracing: Rc<RayTracing>,
        geometry_instance: &GeometryInstance,
    ) -> Result<(Vec<AccelerationStructure>, AccelerationStructure), VulkanError> {
        let command_buffer = self.context.begin_single_time_commands().unwrap();

        let blas = self.create_bottom_level_as(geometry_instance);
        let structure = AccelerationStructureBuilder::new(&self.context, Rc::clone(&ray_tracing))
            .with_bottom_level_as(&[blas])
            .with_command_buffer(command_buffer)
            .build()?;
        let bottom_level_as = vec![structure];

        let instances: Vec<Instance> = bottom_level_as
            .iter()
            .enumerate()
            .map(|(index, blas)| Instance {
                bottom_level_as: blas.get(),
                transform: geometry_instance.transform,
                instance_id: index as u32,
                hit_group_index: (index * 2) as u32,
            })
            .collect();

        let top_level_as =
            AccelerationStructureBuilder::new(&self.context, Rc::clone(&ray_tracing))
                .with_top_level_as(&instances)
                .with_command_buffer(command_buffer)
                .build()?;

        self.context.end_single_time_commands(command_buffer)?;

        Ok((bottom_level_as, top_level_as))
    }

    fn create_bottom_level_as(&self, geom: &GeometryInstance) -> BottomLevelAccelerationStructure {
        BottomLevelAccelerationStructureBuilder::new()
            .with_vertex_buffer(geom.vertex_buffer.get())
            .with_vertex_offset(geom.vertex_offset)
            .with_vertex_count(geom.vertex_count)
            .with_vertex_size(mem::size_of::<Vertex>() as u32)
            .with_index_buffer(geom.index_buffer.get())
            .with_index_offset(geom.index_offset)
            .with_index_count(geom.index_count)
            .with_opaque(true)
            .build()
    }

    fn create_descriptor_set(
        &self,
        geometry_instance: &GeometryInstance,
    ) -> Result<DescriptorSet, VulkanError> {
        DescriptorSetBuilder::new(&self.context, geometry_instance).build()
    }

    fn create_pipeline(
        &self,
        ray_tracing: &RayTracing,
        descriptor_set: &DescriptorSet,
    ) -> Result<Pipeline, VulkanError> {
        let ray_gen_module = ShaderModuleBuilder::new(Rc::clone(&self.context.get_device()))
            .with_path(Path::new("assets/shaders/raygen.spv"))
            .build()?;
        let miss_module = ShaderModuleBuilder::new(Rc::clone(&self.context.get_device()))
            .with_path(Path::new("assets/shaders/miss.spv"))
            .build()?;
        let shadow_miss_module = ShaderModuleBuilder::new(Rc::clone(&self.context.get_device()))
            .with_path(Path::new("assets/shaders/shadow_miss.spv"))
            .build()?;
        let closest_hit_module = ShaderModuleBuilder::new(Rc::clone(&self.context.get_device()))
            .with_path(Path::new("assets/shaders/closesthit.spv"))
            .build()?;

        PipelineBuilder::new(&self.context, ray_tracing, descriptor_set)
            .with_ray_gen_shader(ray_gen_module)
            .with_miss_shader(miss_module)
            .with_shadow_miss_shader(shadow_miss_module)
            .with_hit_shader(closest_hit_module)
            .with_max_recursion_depth(2)
            .build()
    }

    fn create_shader_binding_table(
        &self,
        ray_tracing: &RayTracing,
        pipeline: &Pipeline,
    ) -> Result<ShaderBindingTable, VulkanError> {
        ShaderBindingTableBuilder::new(&self.context, ray_tracing, pipeline).build()
    }
}
