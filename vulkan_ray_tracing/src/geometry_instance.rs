use std::mem;
use std::os::raw::c_void;

use ash::vk;
use nalgebra_glm as glm;
use vulkan_bootstrap::buffer::{Buffer, BufferBuilder, BufferType};
use vulkan_bootstrap::errors::VulkanError;
use vulkan_bootstrap::texture::{Texture, TextureBuilder};
use vulkan_bootstrap::vulkan_context::VulkanContext;

pub struct ImageBuffer {
    pub pixels: Vec<u8>,
    pub tex_width: u32,
    pub tex_height: u32,
    pub tex_channels: u32,
}

#[repr(C, packed)]
pub struct Vertex {
    pub pos: glm::Vec3,
    pub nrm: glm::Vec3,
    pub color: glm::Vec3,
    pub tex_coord: glm::Vec2,
    pub mat_id: i32,
}

impl Vertex {
    pub fn get_binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(mem::size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 5] {
        [
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(memoffset::offset_of!(Vertex, pos) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(memoffset::offset_of!(Vertex, nrm) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(memoffset::offset_of!(Vertex, color) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(3)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(memoffset::offset_of!(Vertex, tex_coord) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(4)
                .format(vk::Format::R32_SINT)
                .offset(memoffset::offset_of!(Vertex, mat_id) as u32)
                .build(),
        ]
    }
}

#[repr(C, packed)]
pub struct Material {
    pub ambient: glm::Vec3,
    pub diffuse: glm::Vec3,
    pub specular: glm::Vec3,
    pub transmittance: glm::Vec3,
    pub emission: glm::Vec3,
    pub shininess: f32,
    pub ior: f32,
    pub dissolve: f32,
    pub illum: i32,
    pub texture_id: i32,
}

impl Default for Material {
    fn default() -> Self {
        Material {
            ambient: glm::vec3(0.1, 0.1, 0.1),
            diffuse: glm::vec3(0.7, 0.7, 0.7),
            specular: glm::vec3(1.0, 1.0, 1.0),
            transmittance: glm::vec3(0.0, 0.0, 0.0),
            emission: glm::vec3(0.0, 0.0, 0.1),
            shininess: 0.0,
            ior: 1.0,
            dissolve: 1.0,
            illum: 0,
            texture_id: -1,
        }
    }
}

pub struct GeometryInstance {
    pub vertex_buffer: Buffer,
    pub vertex_count: u32,
    pub vertex_offset: u32,
    pub index_buffer: Buffer,
    pub index_count: u32,
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
    textures: Vec<ImageBuffer>,
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

    pub fn with_textures(mut self, textures: &mut Vec<ImageBuffer>) -> Self {
        self.textures.append(textures);
        self
    }

    pub fn build(self) -> Result<GeometryInstance, VulkanError> {
        let transform = glm::identity();

        let vertex_buffer = self.create_vertex_buffer(&self.vertices)?;
        let index_buffer = self.create_index_buffer(&self.indices)?;
        let material_buffer = self.create_material_buffer(&self.materials)?;
        let textures = self.create_texture_images(&self.textures)?;

        Ok(GeometryInstance {
            vertex_buffer,
            vertex_count: self.vertices.len() as u32,
            vertex_offset: 0,
            index_buffer,
            index_count: self.indices.len() as u32,
            index_offset: 0,
            material_buffer,
            textures,
            transform,
        })
    }

    fn create_vertex_buffer(&self, vertices: &[Vertex]) -> Result<Buffer, VulkanError> {
        let size = (mem::size_of::<Vertex>() * vertices.len()) as vk::DeviceSize;
        let vertices = vertices.as_ptr() as *const c_void;
        self.create_buffer(BufferType::Vertex, size, vertices)
    }

    fn create_index_buffer(&self, indices: &[u32]) -> Result<Buffer, VulkanError> {
        let size = (mem::size_of::<u32>() * indices.len()) as vk::DeviceSize;
        let indices = indices.as_ptr() as *const c_void;
        self.create_buffer(BufferType::Index, size, indices)
    }

    fn create_material_buffer(&self, materials: &[Material]) -> Result<Buffer, VulkanError> {
        let size = (mem::size_of::<Material>() * materials.len()) as vk::DeviceSize;
        let materials = materials.as_ptr() as *const c_void;

        let mat_buffer = BufferBuilder::new(self.context)
            .with_type(BufferType::Storage)
            .with_size(size)
            .build()?;
        mat_buffer.copy_data(materials)?;

        Ok(mat_buffer)
    }

    fn create_texture_images(&self, images: &[ImageBuffer]) -> Result<Vec<Texture>, VulkanError> {
        let mut textures = vec![];

        if images.is_empty() {
            let image = ImageBuffer {
                pixels: vec![255, 0, 255, 255],
                tex_width: 1,
                tex_height: 1,
                tex_channels: 4,
            };

            let texture = TextureBuilder::new(self.context)
                .with_width(image.tex_width)
                .with_height(image.tex_height)
                .with_pixels(&image.pixels)
                .build()?;
            textures.push(texture);
        }

        for image in images {
            let texture = TextureBuilder::new(self.context)
                .with_width(image.tex_width)
                .with_height(image.tex_height)
                .with_pixels(&image.pixels)
                .build()?;
            textures.push(texture);
        }

        Ok(textures)
    }

    fn create_buffer(
        &self,
        ty: BufferType,
        size: vk::DeviceSize,
        data: *const c_void,
    ) -> Result<Buffer, VulkanError> {
        let staging_buffer = BufferBuilder::new(self.context)
            .with_type(BufferType::Staging)
            .with_size(size)
            .build()?;

        staging_buffer.copy_data(data)?;

        let buffer = BufferBuilder::new(self.context)
            .with_type(ty)
            .with_size(size)
            .build()?;

        self.copy_buffer(staging_buffer.get(), buffer.get(), size)?;

        Ok(buffer)
    }

    fn copy_buffer(
        &self,
        src_buffer: vk::Buffer,
        dst_buffer: vk::Buffer,
        size: vk::DeviceSize,
    ) -> Result<(), VulkanError> {
        let command_buffer = self.context.begin_single_time_commands()?;
        let copy_region = vk::BufferCopy::builder().size(size).build();
        self.context.get_device().cmd_copy_buffer(
            command_buffer,
            src_buffer,
            dst_buffer,
            &[copy_region],
        );
        self.context.end_single_time_commands(command_buffer)
    }
}
