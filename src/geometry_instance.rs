use nalgebra_glm as glm;

use vulkan_helpers::buffer::Buffer;
use vulkan_helpers::vulkan_context::VulkanContext;

use crate::model::Model;

pub struct GeometryInstance {
    pub vertex_buffer: Buffer,
    pub vertex_count: usize,
    pub vertex_offset: u32,
    pub index_buffer: Buffer,
    pub index_count: usize,
    pub index_offset: u32,
    pub transform: glm::Mat4,
}

pub struct GeometryInstanceBuilder<'a> {
    context: &'a VulkanContext,
    model: Option<&'a Model>,
}

impl<'a> GeometryInstanceBuilder<'a> {
    pub fn new(context: &'a VulkanContext) -> Self {
        GeometryInstanceBuilder {
            context,
            model: None,
        }
    }

    pub fn with_model(mut self, model: &'a Model) -> Self {
        self.model = Some(model);
        self
    }

    pub fn build(self) -> GeometryInstance {
        let transform = glm::identity();
        let model = self.model.unwrap();

        let vertex_buffer = self.context.create_vertex_buffer(&model.vertices).unwrap();
        let index_buffer = self.context.create_index_buffer(&model.indices).unwrap();

        GeometryInstance {
            vertex_buffer,
            vertex_count: model.vertices.len(),
            vertex_offset: 0,
            index_buffer,
            index_count: model.indices.len(),
            index_offset: 0,
            transform,
        }
    }
}
