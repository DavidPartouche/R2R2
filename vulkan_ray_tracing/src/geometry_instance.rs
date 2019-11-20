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

pub struct GeometryInstance {
    pub vertex_buffer: Buffer,
    pub vertex_count: u32,
    pub vertex_offset: u32,
    pub vertex_size: u32,
    pub index_buffer: Buffer,
    pub index_count: u32,
    pub index_offset: u32,
    pub material_buffer: Buffer,
    //    pub textures: Vec<Texture>,
    pub transform: glm::Mat4,
}

pub struct GeometryInstanceBuilder<'a> {
    context: &'a VulkanContext,
    vertices: Option<&'a [u8]>,
    vertices_count: usize,
    indices: Option<&'a [u32]>,
    materials: Option<&'a [u8]>,
    textures: Vec<ImageBuffer>,
}

impl<'a> GeometryInstanceBuilder<'a> {
    pub fn new(context: &'a VulkanContext) -> Self {
        GeometryInstanceBuilder {
            context,
            vertices: None,
            vertices_count: 0,
            indices: None,
            materials: None,
            textures: vec![],
        }
    }

    pub fn with_vertices(mut self, vertices: &'a [u8], vertices_count: usize) -> Self {
        self.vertices = Some(vertices);
        self.vertices_count = vertices_count;
        self
    }

    pub fn with_indices(mut self, indices: &'a [u32]) -> Self {
        self.indices = Some(indices);
        self
    }

    pub fn with_materials(mut self, materials: &'a [u8]) -> Self {
        self.materials = Some(materials);
        self
    }

    pub fn with_textures(mut self, textures: &mut Vec<ImageBuffer>) -> Self {
        self.textures.append(textures);
        self
    }

    pub fn build(self) -> Result<GeometryInstance, VulkanError> {
        let transform = glm::identity();

        let vertex_buffer = self.create_vertex_buffer()?;
        let index_buffer = self.create_index_buffer()?;
        let material_buffer = self.create_material_buffer()?;
        //        let textures = self.create_texture_images(&self.textures)?;

        Ok(GeometryInstance {
            vertex_buffer,
            vertex_count: self.vertices_count as u32,
            vertex_offset: 0,
            vertex_size: (self.vertices.unwrap().len() / self.vertices_count) as u32,
            index_buffer,
            index_count: self.indices.unwrap().len() as u32,
            index_offset: 0,
            material_buffer,
            //            textures,
            transform,
        })
    }

    fn create_vertex_buffer(&self) -> Result<Buffer, VulkanError> {
        let vertices = self.vertices.unwrap();
        let data = vertices.as_ptr() as *const c_void;
        self.create_buffer(BufferType::Vertex, vertices.len() as vk::DeviceSize, data)
    }

    fn create_index_buffer(&self) -> Result<Buffer, VulkanError> {
        let indices = self.indices.unwrap();
        let size = (mem::size_of::<u32>() * indices.len()) as vk::DeviceSize;
        let data = indices.as_ptr() as *const c_void;
        self.create_buffer(BufferType::Index, size, data)
    }

    fn create_material_buffer(&self) -> Result<Buffer, VulkanError> {
        let materials = self.materials.unwrap();
        let data = materials.as_ptr() as *const c_void;

        let mat_buffer = BufferBuilder::new(self.context)
            .with_type(BufferType::Storage)
            .with_size(materials.len() as vk::DeviceSize)
            .build()?;
        mat_buffer.copy_data(data)?;

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
