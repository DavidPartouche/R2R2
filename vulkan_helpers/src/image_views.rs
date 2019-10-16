use std::rc::Rc;

use ash::vk;

use crate::device::Device;
use crate::errors::VulkanError;
use crate::surface_format::SurfaceFormat;
use crate::swapchain::Swapchain;

pub struct ImageViews {
    device: Rc<Device>,
    back_buffer_views: Vec<vk::ImageView>,
}

impl Drop for ImageViews {
    fn drop(&mut self) {
        for back_buffer_view in self.back_buffer_views.iter() {
            self.device.destroy_image_view(*back_buffer_view);
        }
    }
}

impl ImageViews {
    pub fn get_image_views(&self) -> &Vec<vk::ImageView> {
        &self.back_buffer_views
    }
}

pub struct ImageViewsBuilder<'a> {
    device: Rc<Device>,
    surface_format: SurfaceFormat,
    swapchain: &'a Swapchain,
}

impl<'a> ImageViewsBuilder<'a> {
    pub fn new(
        device: Rc<Device>,
        surface_format: SurfaceFormat,
        swapchain: &'a Swapchain,
    ) -> Self {
        ImageViewsBuilder {
            device,
            surface_format,
            swapchain,
        }
    }

    pub fn build(self) -> Result<ImageViews, VulkanError> {
        let mut back_buffer_views = vec![];

        for back_buffer in self.swapchain.get_back_buffers() {
            let view_info = vk::ImageViewCreateInfo::builder()
                .image(*back_buffer)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(self.surface_format.format)
                .components(
                    vk::ComponentMapping::builder()
                        .r(vk::ComponentSwizzle::R)
                        .g(vk::ComponentSwizzle::G)
                        .b(vk::ComponentSwizzle::B)
                        .a(vk::ComponentSwizzle::A)
                        .build(),
                )
                .subresource_range(
                    vk::ImageSubresourceRange::builder()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1)
                        .build(),
                )
                .build();

            back_buffer_views.push(self.device.create_image_view(&view_info)?);
        }

        Ok(ImageViews {
            device: self.device,
            back_buffer_views,
        })
    }
}
