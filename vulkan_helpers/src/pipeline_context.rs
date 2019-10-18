use std::os::raw::c_void;
use std::path::Path;
use std::rc::Rc;

use ash::vk;

use crate::buffer::{Buffer, BufferBuilder, BufferType};
use crate::descriptor_set::{DescriptorSet, DescriptorSetBuilder};
use crate::descriptor_set_layout::{DescriptorSetLayout, DescriptorSetLayoutBuilder};
use crate::device::Device;
use crate::errors::VulkanError;
use crate::pipeline::{Pipeline, PipelineBuilder};
use crate::texture::Texture;
use crate::vulkan_context::VulkanContext;

pub struct GraphicsPipelineContext {
    device: Rc<Device>,
    uniform_buffer: Buffer,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    material_buffer: Buffer,
    descriptor_set: DescriptorSet,
    graphics_pipeline: Pipeline,
    descriptor_set_layout: DescriptorSetLayout,
    indices_count: u32,
}

impl Drop for GraphicsPipelineContext {
    fn drop(&mut self) {
        self.device.queue_wait_idle().unwrap();
    }
}

impl GraphicsPipelineContext {
    pub fn update_uniform_buffer(&self, ubo: *const c_void) -> Result<(), VulkanError> {
        self.uniform_buffer.copy_data(ubo)
    }

    pub fn draw(&self, command_buffer: vk::CommandBuffer) {
        self.device
            .cmd_bind_pipeline(command_buffer, self.graphics_pipeline.get());
        self.device.cmd_bind_descriptor_sets(
            command_buffer,
            self.graphics_pipeline.get_layout(),
            &[self.descriptor_set.get()],
        );
        self.device
            .cmd_bind_vertex_buffers(command_buffer, &[self.vertex_buffer.get()]);
        self.device
            .cmd_bind_index_buffer(command_buffer, self.index_buffer.get());
        self.device
            .cmd_draw_indexed(command_buffer, self.indices_count);

        self.device.cmd_next_subpass(command_buffer);
    }
}

pub struct GraphicsPipelineContextBuilder<'a> {
    context: &'a VulkanContext,
    vertex_shader: Option<&'a Path>,
    fragment_shader: Option<&'a Path>,
    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
    material_buffer: Option<Buffer>,
    textures: Vec<Texture>,
    ubo_size: usize,
    indices_count: usize,
}

impl<'a> GraphicsPipelineContextBuilder<'a> {
    pub fn new(context: &'a VulkanContext) -> Self {
        GraphicsPipelineContextBuilder {
            context,
            vertex_shader: None,
            fragment_shader: None,
            vertex_buffer: None,
            index_buffer: None,
            material_buffer: None,
            textures: vec![],
            ubo_size: 0,
            indices_count: 0,
        }
    }

    pub fn with_vertex_shader(mut self, vertex_shader: &'a Path) -> Self {
        self.vertex_shader = Some(vertex_shader);
        self
    }

    pub fn with_fragment_shader(mut self, fragment_shader: &'a Path) -> Self {
        self.fragment_shader = Some(fragment_shader);
        self
    }

    pub fn with_ubo_size(mut self, size: usize) -> Self {
        self.ubo_size = size;
        self
    }

    pub fn with_vertex_buffer(mut self, vertex_buffer: Buffer) -> Self {
        self.vertex_buffer = Some(vertex_buffer);
        self
    }

    pub fn with_index_buffer(mut self, index_buffer: Buffer) -> Self {
        self.index_buffer = Some(index_buffer);
        self
    }

    pub fn with_material_buffer(mut self, material_buffer: Buffer) -> Self {
        self.material_buffer = Some(material_buffer);
        self
    }

    pub fn with_textures(mut self, textures: Vec<Texture>) -> Self {
        self.textures = textures;
        self
    }

    pub fn with_indices_count(mut self, indices_count: usize) -> Self {
        self.indices_count = indices_count;
        self
    }

    pub fn build(self) -> Result<GraphicsPipelineContext, VulkanError> {
        let descriptor_set_layout = DescriptorSetLayoutBuilder::new(self.context)
            .with_texture_count(self.textures.len() as u32)
            .build()?;

        let graphics_pipeline = PipelineBuilder::new(self.context, &descriptor_set_layout)
            .with_vertex_shader(self.vertex_shader.unwrap())
            .with_fragment_shader(self.fragment_shader.unwrap())
            .build()?;

        let uniform_buffer = BufferBuilder::new(self.context)
            .with_type(BufferType::Uniform)
            .with_size(self.ubo_size as vk::DeviceSize)
            .build()?;

        let material_buffer = self.material_buffer.ok_or_else(|| {
            VulkanError::GrahpicsPipelineCreationError(String::from("Material buffer missing"))
        })?;

        let descriptor_set = DescriptorSetBuilder::new(self.context, &descriptor_set_layout)
            .with_uniform_buffer(&uniform_buffer)
            .with_material_buffer(&material_buffer)
            .with_textures(self.textures)
            .build()
            .unwrap();

        Ok(GraphicsPipelineContext {
            device: Rc::clone(&self.context.device),
            descriptor_set_layout,
            graphics_pipeline,
            uniform_buffer,
            vertex_buffer: self.vertex_buffer.unwrap(),
            index_buffer: self.index_buffer.unwrap(),
            material_buffer,
            indices_count: self.indices_count as u32,
            descriptor_set,
        })
    }
}
