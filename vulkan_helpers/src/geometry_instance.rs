use crate::buffer::Buffer;
use crate::errors::VulkanError;
use crate::glm;
use crate::images::Image;
use crate::material::Material;
use crate::texture::Texture;
use crate::vertex::Vertex;
use crate::vulkan_context::VulkanContext;

#[repr(C, packed)]
pub struct UniformBufferObject {
    model: glm::Mat4,
    view: glm::Mat4,
    proj: glm::Mat4,
    model_it: glm::Mat4,
}

pub struct GeometryInstance {
    pub vertex_buffer: Buffer,
    pub vertex_count: usize,
    pub vertex_offset: u32,
    pub index_buffer: Buffer,
    pub index_count: usize,
    pub index_offset: u32,
    pub material_buffer: Buffer,
    pub textures: Vec<Texture>,
    pub transform: glm::Mat4,
}

pub struct GeometryInstanceBuilder<'a> {
    context: &'a VulkanContext,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    materials: Vec<Material>,
    textures: Vec<Image>,
}

impl<'a> GeometryInstanceBuilder<'a> {
    pub fn new(context: &'a VulkanContext) -> Self {
        GeometryInstanceBuilder {
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

    pub fn build(self) -> Result<GeometryInstance, VulkanError> {
        let transform = glm::identity();

        let vertex_buffer = self.context.create_vertex_buffer(&self.vertices)?;
        let index_buffer = self.context.create_index_buffer(&self.indices)?;
        let material_buffer = self.context.create_material_buffer(&self.materials)?;
        let textures = self.context.create_texture_images(&self.textures)?;

        Ok(GeometryInstance {
            vertex_buffer,
            vertex_count: self.vertices.len(),
            vertex_offset: 0,
            index_buffer,
            index_count: self.indices.len(),
            index_offset: 0,
            material_buffer,
            textures,
            transform,
        })
    }
}
