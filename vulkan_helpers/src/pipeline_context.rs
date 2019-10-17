use std::path::Path;
use std::rc::Rc;

use ash::vk;

use crate::buffer::{Buffer, BufferBuilder};
use crate::descriptor_set::{DescriptorSet, DescriptorSetBuilder};
use crate::descriptor_set_layout::{DescriptorSetLayout, DescriptorSetLayoutBuilder};
use crate::device::Device;
use crate::pipeline::{Pipeline, PipelineBuilder};
use crate::vulkan_context::VulkanContext;

pub struct GraphicsPipelineContext {
    device: Rc<Device>,
    descriptor_set: DescriptorSet,
    uniform_buffer: Buffer,
    graphics_pipeline: Pipeline,
    descriptor_set_layout: DescriptorSetLayout,
}

impl Drop for GraphicsPipelineContext {
    fn drop(&mut self) {
        self.device.queue_wait_idle().unwrap();
    }
}

impl GraphicsPipelineContext {
    pub fn get(&self) -> &Pipeline {
        &self.graphics_pipeline
    }
}

pub struct GraphicsPipelineContextBuilder<'a> {
    context: &'a VulkanContext,
    texture_count: u32,
    vertex_shader: Option<&'a Path>,
    fragment_shader: Option<&'a Path>,
    ubo_size: usize,
}

impl<'a> GraphicsPipelineContextBuilder<'a> {
    pub fn new(context: &'a VulkanContext) -> Self {
        GraphicsPipelineContextBuilder {
            context,
            texture_count: 0,
            vertex_shader: None,
            fragment_shader: None,
            ubo_size: 0,
        }
    }

    pub fn with_texture_count(mut self, texture_count: u32) -> Self {
        self.texture_count = texture_count;
        self
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

    pub fn build(self) -> GraphicsPipelineContext {
        let descriptor_set_layout = DescriptorSetLayoutBuilder::new(self.context)
            .with_texture_count(self.texture_count)
            .build()
            .unwrap();

        let graphics_pipeline = PipelineBuilder::new(self.context, &descriptor_set_layout)
            .with_vertex_shader(self.vertex_shader.unwrap())
            .with_fragment_shader(self.fragment_shader.unwrap())
            .build()
            .unwrap();

        let uniform_buffer = BufferBuilder::new(self.context)
            .with_size(self.ubo_size as vk::DeviceSize)
            .build()
            .unwrap();

        let descriptor_set =
            DescriptorSetBuilder::new(self.context, &descriptor_set_layout, &uniform_buffer)
                .build()
                .unwrap();

        GraphicsPipelineContext {
            device: Rc::clone(&self.context.device),
            descriptor_set_layout,
            graphics_pipeline,
            uniform_buffer,
            descriptor_set,
        }
    }
}
