use std::mem;
use std::path::Path;
use std::rc::Rc;

use ash::vk;

use crate::acceleration_structure::{
    AccelerationStructure, AccelerationStructureBuilder, Instance,
};
use crate::bottom_level_acceleration_structure::{
    BottomLevelAccelerationStructure, BottomLevelAccelerationStructureBuilder,
};
use crate::buffer::{Buffer, BufferBuilder, BufferType};
use crate::descriptor_set::{DescriptorSet, DescriptorSetBuilder};
use crate::errors::VulkanError;
use crate::geometry_instance::{
    GeometryInstance, GeometryInstanceBuilder, UniformBufferObject, Vertex,
};
use crate::images::Image;
use crate::material::Material;
use crate::pipeline::{Pipeline, PipelineBuilder};
use crate::ray_tracing::{RayTracing, RayTracingBuilder};
use crate::shader_module::ShaderModuleBuilder;
use crate::vulkan_context::VulkanContext;

pub struct RayTracingPipeline {
    _pipeline: Pipeline,
    descriptor_set: DescriptorSet,
    _top_level_as: AccelerationStructure,
    _bottom_level_as: Vec<AccelerationStructure>,
    _geometry_instance: GeometryInstance,
    _camera_buffer: Buffer,
    _ray_tracing: Rc<RayTracing>,
}

impl RayTracingPipeline {
    pub fn draw(&self) {
        self.descriptor_set
            .update_render_target(vk::ImageView::null());
    }
}

pub struct RayTracingPipelineBuilder<'a> {
    context: &'a VulkanContext,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    materials: Vec<Material>,
    textures: Vec<Image>,
}

impl<'a> RayTracingPipelineBuilder<'a> {
    pub fn new(context: &'a VulkanContext) -> Self {
        RayTracingPipelineBuilder {
            context,
            vertices: vec![],
            indices: vec![],
            materials: vec![],
            textures: vec![],
        }
    }

    pub fn with_vertices(mut self, vertices: &mut Vec<Vertex>) -> Self {
        self.vertices.append(vertices);
        self
    }

    pub fn with_indices(mut self, indices: &mut Vec<u32>) -> Self {
        self.indices.append(indices);
        self
    }

    pub fn with_materials(mut self, materials: &mut Vec<Material>) -> Self {
        self.materials.append(materials);
        self
    }

    pub fn with_textures(mut self, textures: &mut Vec<Image>) -> Self {
        self.textures.append(textures);
        self
    }

    pub fn build(mut self) -> Result<RayTracingPipeline, VulkanError> {
        let ray_tracing = Rc::new(RayTracingBuilder::new(&self.context).build()?);

        let camera_buffer = BufferBuilder::new(&self.context)
            .with_type(BufferType::Uniform)
            .with_size(mem::size_of::<UniformBufferObject>() as u64)
            .build()?;

        let geometry_instance = GeometryInstanceBuilder::new(&self.context)
            .with_vertices(&mut self.vertices)
            .with_indices(&mut self.indices)
            .with_materials(&mut self.materials)
            .with_textures(&mut self.textures)
            .build()?;

        let (bottom_level_as, top_level_as) =
            self.create_acceleration_structures(Rc::clone(&ray_tracing), &geometry_instance)?;

        let descriptor_set =
            self.create_descriptor_set(&camera_buffer, &geometry_instance, &top_level_as)?;

        let pipeline = self.create_pipeline(&ray_tracing, &descriptor_set)?;

        Ok(RayTracingPipeline {
            _ray_tracing: ray_tracing,
            _camera_buffer: camera_buffer,
            _geometry_instance: geometry_instance,
            _bottom_level_as: bottom_level_as,
            _top_level_as: top_level_as,
            descriptor_set,
            _pipeline: pipeline,
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
                hit_group_index: index as u32,
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
            .with_vertex_count(geom.vertex_count as u32)
            .with_vertex_size(mem::size_of::<Vertex>() as u32)
            .with_index_buffer(geom.index_buffer.get())
            .with_index_offset(geom.index_offset)
            .with_index_count(geom.index_count as u32)
            .build()
    }

    fn create_descriptor_set(
        &self,
        camera_buffer: &Buffer,
        geometry_instance: &GeometryInstance,
        top_level_as: &AccelerationStructure,
    ) -> Result<DescriptorSet, VulkanError> {
        DescriptorSetBuilder::new(
            &self.context,
            camera_buffer,
            geometry_instance,
            top_level_as,
        )
        .build()
    }

    fn create_pipeline(
        &self,
        ray_tracing: &RayTracing,
        descriptor_set: &DescriptorSet,
    ) -> Result<Pipeline, VulkanError> {
        let ray_gen_module = ShaderModuleBuilder::new(Rc::clone(&self.context.device))
            .with_path(Path::new("assets/shaders/raygen.spv"))
            .build()?;
        let miss_module = ShaderModuleBuilder::new(Rc::clone(&self.context.device))
            .with_path(Path::new("assets/shaders/miss.spv"))
            .build()?;
        let closest_hit_module = ShaderModuleBuilder::new(Rc::clone(&self.context.device))
            .with_path(Path::new("assets/shaders/closesthit.spv"))
            .build()?;

        PipelineBuilder::new(&self.context, ray_tracing, descriptor_set)
            .with_ray_gen_shader(ray_gen_module)
            .with_miss_shader(miss_module)
            .with_closest_hit_shader(closest_hit_module)
            .with_max_recursion_depth(1)
            .build()
    }
}
