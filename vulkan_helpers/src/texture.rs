use std::os::raw::c_void;
use std::rc::Rc;

use ash::vk;

use crate::buffer::{BufferBuilder, BufferType};
use crate::device::VulkanDevice;
use crate::errors::VulkanError;
use crate::images;
use crate::images::Image;
use crate::vulkan_context::VulkanContext;

pub struct Texture {
    device: Rc<VulkanDevice>,
    texture_image: vk::Image,
    texture_image_memory: vk::DeviceMemory,
    texture_image_view: vk::ImageView,
    texture_sampler: vk::Sampler,
}

impl Drop for Texture {
    fn drop(&mut self) {
        self.device.destroy_sampler(self.texture_sampler);
        self.device.destroy_image_view(self.texture_image_view);
        self.device.destroy_image(self.texture_image);
        self.device.free_memory(self.texture_image_memory);
    }
}

impl Texture {
    pub fn get_image_view(&self) -> vk::ImageView {
        self.texture_image_view
    }

    pub fn get_sampler(&self) -> vk::Sampler {
        self.texture_sampler
    }
}

pub struct TextureBuilder<'a> {
    context: &'a VulkanContext,
    image: Option<&'a Image>,
}

impl<'a> TextureBuilder<'a> {
    pub fn new(context: &'a VulkanContext) -> Self {
        TextureBuilder {
            context,
            image: None,
        }
    }

    pub fn with_image(mut self, image: &'a Image) -> Self {
        self.image = Some(image);
        self
    }

    pub fn build(self) -> Result<Texture, VulkanError> {
        let image = self
            .image
            .ok_or_else(|| VulkanError::TextureCreationError(String::from("No image provided")))?;

        let image_size = (image.tex_width * image.tex_height * 4) as vk::DeviceSize;
        let data = image.pixels.as_ptr() as *const c_void;

        let staging_buffer = BufferBuilder::new(self.context)
            .with_type(BufferType::Staging)
            .with_size(image_size)
            .build()?;

        staging_buffer.copy_data(data)?;

        let (texture_image, texture_image_memory) = images::create_image(
            &self.context.instance,
            &self.context.device,
            &self.context.physical_device,
            image.tex_width,
            image.tex_height,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        images::transition_image_layout(
            &self.context.device,
            &self.context.command_buffers,
            texture_image,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        )?;

        self.copy_buffer_to_image(
            staging_buffer.get(),
            texture_image,
            image.tex_width,
            image.tex_height,
        )?;

        images::transition_image_layout(
            &self.context.device,
            &self.context.command_buffers,
            texture_image,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        )?;

        let texture_image_view = images::create_image_view(
            &self.context.device,
            texture_image,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageAspectFlags::COLOR,
        )?;

        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(true)
            .max_anisotropy(16.0)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .build();

        let texture_sampler = self.context.device.create_sampler(&sampler_info)?;

        Ok(Texture {
            device: Rc::clone(&self.context.device),
            texture_image,
            texture_image_memory,
            texture_image_view,
            texture_sampler,
        })
    }

    fn copy_buffer_to_image(
        &self,
        buffer: vk::Buffer,
        image: vk::Image,
        width: u32,
        height: u32,
    ) -> Result<(), VulkanError> {
        let command_buffer = self.context.command_buffers.begin_single_time_commands(0)?;

        let region = vk::BufferImageCopy::builder()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(
                vk::ImageSubresourceLayers::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(0)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
            )
            .image_offset(vk::Offset3D::builder().x(0).y(0).z(0).build())
            .image_extent(
                vk::Extent3D::builder()
                    .width(width)
                    .height(height)
                    .depth(1)
                    .build(),
            )
            .build();

        self.context.device.cmd_copy_buffer_to_image(
            command_buffer,
            buffer,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[region],
        );

        self.context
            .command_buffers
            .end_single_time_commands(command_buffer, 0)
    }
}
