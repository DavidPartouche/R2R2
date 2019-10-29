use std::ffi::CStr;
use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::rc::Rc;

use ash::vk;
use vulkan_bootstrap::buffer::{Buffer, BufferBuilder, BufferType};
use vulkan_bootstrap::device::VulkanDevice;
use vulkan_bootstrap::errors::VulkanError;
use vulkan_bootstrap::shader_module::ShaderModuleBuilder;
use vulkan_bootstrap::vulkan_context::VulkanContext;

use crate::geometry_instance::{GeometryInstance, UniformBufferObject, Vertex};
use crate::glm;

pub struct GraphicsPipeline {
    device: Rc<VulkanDevice>,
    geometry_instance: GeometryInstance,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    uniform_buffer: Buffer,
    descriptor_set: vk::DescriptorSet,
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        self.device.destroy_pipeline(self.pipeline);
        self.device.destroy_pipeline_layout(self.pipeline_layout);
        self.device
            .destroy_descriptor_set_layout(self.descriptor_set_layout);
        self.device.destroy_descriptor_pool(self.descriptor_pool);
    }
}

impl GraphicsPipeline {
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

        self.uniform_buffer.copy_data(data)
    }

    pub fn draw(&mut self, context: &mut VulkanContext) -> Result<(), VulkanError> {
        context.frame_begin()?;
        let command_buffer = context.get_current_command_buffer();

        context.begin_render_pass();
        context.get_device().cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline,
        );

        context.get_device().cmd_bind_descriptor_sets(
            command_buffer,
            self.pipeline_layout,
            &[self.descriptor_set],
        );

        context.get_device().cmd_bind_vertex_buffers(
            command_buffer,
            &[self.geometry_instance.vertex_buffer.get()],
            &[0],
        );
        context.get_device().cmd_bind_index_buffer(
            command_buffer,
            self.geometry_instance.index_buffer.get(),
            0,
        );
        context
            .get_device()
            .cmd_draw_index(command_buffer, self.geometry_instance.index_count);

        context.get_device().cmd_next_subpass(command_buffer);
        context.end_render_pass();
        context.frame_end()?;
        context.frame_present()
    }
}

pub struct GraphicsPipelineBuilder<'a> {
    context: &'a VulkanContext,
    geometry_instance: Option<GeometryInstance>,
    width: u32,
    height: u32,
}

impl<'a> GraphicsPipelineBuilder<'a> {
    pub fn new(context: &'a VulkanContext) -> Self {
        GraphicsPipelineBuilder {
            context,
            geometry_instance: None,
            width: 0,
            height: 0,
        }
    }

    pub fn with_geometry_instance(mut self, geometry_instance: GeometryInstance) -> Self {
        self.geometry_instance = Some(geometry_instance);
        self
    }

    pub fn with_width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    pub fn with_height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    pub fn build(self) -> Result<GraphicsPipeline, VulkanError> {
        let descriptor_pool = self.create_descriptor_pool()?;
        let descriptor_set_layout = self.create_descriptor_set_layout()?;
        let (pipeline_layout, pipeline) = self.create_pipeline(descriptor_set_layout)?;

        let size = mem::size_of::<UniformBufferObject>() as vk::DeviceSize;
        let uniform_buffer = BufferBuilder::new(self.context)
            .with_size(size)
            .with_type(BufferType::Uniform)
            .build()?;

        let descriptor_set =
            self.update_descriptor_sets(descriptor_pool, descriptor_set_layout, &uniform_buffer)?;

        Ok(GraphicsPipeline {
            device: Rc::clone(self.context.get_device()),
            geometry_instance: self.geometry_instance.unwrap(),
            descriptor_pool,
            descriptor_set_layout,
            pipeline_layout,
            pipeline,
            uniform_buffer,
            descriptor_set,
        })
    }

    fn create_descriptor_set_layout(&self) -> Result<vk::DescriptorSetLayout, VulkanError> {
        let ubo_layout_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .build();

        let ubo_mat_color_layout_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(1)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .build();

        let sampler_layout_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(2)
            .descriptor_count(self.geometry_instance.as_ref().unwrap().textures.len() as u32)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build();

        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&[
                ubo_layout_binding,
                ubo_mat_color_layout_binding,
                sampler_layout_binding,
            ])
            .build();

        self.context
            .get_device()
            .create_descriptor_set_layout(&layout_info)
    }

    fn create_pipeline(
        &self,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> Result<(vk::PipelineLayout, vk::Pipeline), VulkanError> {
        let vert_shader = ShaderModuleBuilder::new(Rc::clone(self.context.get_device()))
            .with_path(Path::new("assets/shaders/vert_shader.spv"))
            .build()?;

        let frag_shader = ShaderModuleBuilder::new(Rc::clone(self.context.get_device()))
            .with_path(Path::new("assets/shaders/frag_shader.spv"))
            .build()?;

        let vert_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader.get())
            .name(CStr::from_bytes_with_nul(b"main\0").unwrap())
            .build();

        let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader.get())
            .name(CStr::from_bytes_with_nul(b"main\0").unwrap())
            .build();

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&[Vertex::get_binding_description()])
            .vertex_attribute_descriptions(&Vertex::get_attribute_descriptions())
            .build();

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false)
            .build();

        let viewport = vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(self.width as f32)
            .height(self.height as f32)
            .min_depth(0.0)
            .max_depth(1.0)
            .build();

        let scissor = vk::Rect2D::builder()
            .offset(vk::Offset2D::builder().x(0).y(0).build())
            .extent(
                vk::Extent2D::builder()
                    .width(self.width)
                    .height(self.height)
                    .build(),
            )
            .build();

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&[viewport])
            .scissors(&[scissor])
            .build();

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .depth_bias_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .build();

        let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .build();

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::all())
            .blend_enable(false)
            .build();

        let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&[color_blend_attachment])
            .blend_constants([0.0, 0.0, 0.0, 0.0])
            .build();

        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .min_depth_bounds(0.0)
            .max_depth_bounds(1.0)
            .stencil_test_enable(false)
            .build();

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&[descriptor_set_layout])
            .build();

        let pipeline_layout = self
            .context
            .get_device()
            .create_pipeline_layout(&pipeline_layout_info)?;

        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&[vert_shader_stage_info, frag_shader_stage_info])
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .depth_stencil_state(&depth_stencil)
            .layout(pipeline_layout)
            .render_pass(self.context.get_render_pass().get())
            .subpass(0)
            .build();

        let pipeline = self
            .context
            .get_device()
            .create_graphics_pipelines(&[pipeline_info])?[0];

        Ok((pipeline_layout, pipeline))
    }

    fn create_descriptor_pool(&self) -> Result<vk::DescriptorPool, VulkanError> {
        let pool_size = [
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1000)
                .build(),
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1000)
                .build(),
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1000)
                .build(),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET)
            .max_sets(1000)
            .pool_sizes(&pool_size)
            .build();

        self.context.get_device().create_descriptor_pool(&pool_info)
    }

    fn update_descriptor_sets(
        &self,
        descriptor_pool: vk::DescriptorPool,
        descriptor_set_layout: vk::DescriptorSetLayout,
        uniform_buffer: &Buffer,
    ) -> Result<vk::DescriptorSet, VulkanError> {
        let alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&[descriptor_set_layout])
            .build();

        let descriptor_set = self
            .context
            .get_device()
            .allocate_descriptor_sets(&alloc_info)?[0];

        let buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(uniform_buffer.get())
            .offset(0)
            .range(vk::WHOLE_SIZE)
            .build();

        let mat_color_buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(
                self.geometry_instance
                    .as_ref()
                    .unwrap()
                    .material_buffer
                    .get(),
            )
            .offset(0)
            .range(vk::WHOLE_SIZE)
            .build();

        let mut image_infos = vec![];
        for texture in self.geometry_instance.as_ref().unwrap().textures.iter() {
            let image_info = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(texture.get_image_view())
                .sampler(texture.get_sampler())
                .build();
            image_infos.push(image_info);
        }

        let mut descriptor_writes = vec![];
        let wds = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_set)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .dst_binding(0)
            .buffer_info(&[buffer_info])
            .build();
        descriptor_writes.push(wds);

        let wds = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_set)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .dst_binding(1)
            .buffer_info(&[mat_color_buffer_info])
            .build();
        descriptor_writes.push(wds);

        let wds = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_set)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .dst_binding(2)
            .image_info(&image_infos)
            .build();
        descriptor_writes.push(wds);

        self.context
            .get_device()
            .update_descriptor_sets(&descriptor_writes);

        Ok(descriptor_set)
    }
}
