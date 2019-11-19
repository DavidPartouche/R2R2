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

#[repr(C)]
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
    pub vertex_size: u32,
    pub index_buffer: Buffer,
    pub index_count: u32,
    pub index_offset: u32,
    //    pub material_buffer: Buffer,
    //    pub textures: Vec<Texture>,
    pub transform: glm::Mat4,
}

pub struct GeometryInstanceBuilder<'a> {
    context: &'a VulkanContext,
    vertices: Option<&'a [u8]>,
    vertices_count: usize,
    indices: Option<&'a [u16]>,
    materials: Vec<Material>,
    textures: Vec<ImageBuffer>,
}

impl<'a> GeometryInstanceBuilder<'a> {
    pub fn new(context: &'a VulkanContext) -> Self {
        GeometryInstanceBuilder {
            context,
            vertices: None,
            vertices_count: 0,
            indices: None,
            materials: vec![],
            textures: vec![],
        }
    }

    pub fn with_vertices(mut self, vertices: &'a [u8], vertices_count: usize) -> Self {
        self.vertices = Some(vertices);
        self.vertices_count = vertices_count;
        self
    }

    pub fn with_indices(mut self, indices: &'a [u16]) -> Self {
        self.indices = Some(indices);
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

        let vertex_buffer = self.create_vertex_buffer(&self.vertices.unwrap())?;
        let index_buffer = self.create_index_buffer(&self.indices.unwrap())?;
        //        let material_buffer = self.create_material_buffer(&self.materials)?;
        //        let textures = self.create_texture_images(&self.textures)?;

        Ok(GeometryInstance {
            vertex_buffer,
            vertex_count: self.vertices_count as u32,
            vertex_offset: 0,
            vertex_size: (self.vertices.unwrap().len() / self.vertices_count) as u32,
            index_buffer,
            index_count: self.indices.unwrap().len() as u32,
            index_offset: 0,
            //            material_buffer,
            //            textures,
            transform,
        })
    }

    fn create_vertex_buffer(&self, vertices: &[u8]) -> Result<Buffer, VulkanError> {
        let data = vertices.as_ptr() as *const c_void;
        self.create_buffer(BufferType::Vertex, vertices.len() as vk::DeviceSize, data)
    }

    fn create_index_buffer(&self, indices: &[u16]) -> Result<Buffer, VulkanError> {
        let size = (mem::size_of::<u16>() * indices.len()) as vk::DeviceSize;
        let data = indices.as_ptr() as *const c_void;
        self.create_buffer(BufferType::Index, size, data)
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
