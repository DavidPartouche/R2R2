use ash::extensions::khr;
use ash::vk;

use crate::device::Device;
use crate::errors::VulkanError;
use crate::physical_device::PhysicalDevice;
use crate::present_mode::PresentMode;
use crate::surface::Surface;
use crate::surface_format::SurfaceFormat;

pub struct Swapchain {
    swapchain_loader: khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    back_buffers: Vec<vk::Image>,
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
        }
    }
}

impl Swapchain {
    pub fn get_back_buffers(&self) -> &Vec<vk::Image> {
        &self.back_buffers
    }
}

pub struct SwapchainBuilder<'a> {
    device: &'a Device,
    surface: &'a Surface,
    physical_device: PhysicalDevice,
    surface_format: SurfaceFormat,
    present_mode: PresentMode,
    width: u32,
    height: u32,
}

impl<'a> SwapchainBuilder<'a> {
    pub fn new(
        device: &'a Device,
        surface: &'a Surface,
        physical_device: PhysicalDevice,
        surface_format: SurfaceFormat,
        present_mode: PresentMode,
    ) -> Self {
        SwapchainBuilder {
            device,
            surface,
            physical_device,
            surface_format,
            present_mode,
            width: 0,
            height: 0,
        }
    }

    pub fn with_width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    pub fn with_height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    pub fn build(self) -> Result<Swapchain, VulkanError> {
        let cap = self
            .surface
            .get_physical_device_surface_capabilities(self.physical_device)?;

        let image_count = if cap.max_image_count > 0 {
            cap.max_image_count.min(cap.min_image_count + 2)
        } else {
            cap.min_image_count + 2
        };

        let (width, height) = if cap.current_extent.width == std::u32::MAX {
            (self.width, self.height)
        } else {
            (cap.current_extent.width, cap.current_extent.height)
        };

        let info = vk::SwapchainCreateInfoKHR::builder()
            .surface(self.surface.get())
            .image_format(self.surface_format.format)
            .image_color_space(self.surface_format.color_space)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::STORAGE)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(self.present_mode)
            .clipped(true)
            .min_image_count(image_count)
            .image_extent(vk::Extent2D::builder().width(width).height(height).build())
            .build();

        let swapchain_loader = self.device.new_swapchain();
        let swapchain = unsafe { swapchain_loader.create_swapchain(&info, None) }
            .map_err(|err| VulkanError::SwapchainCreationError(err.to_string()))?;

        let back_buffer = unsafe { swapchain_loader.get_swapchain_images(swapchain) }
            .map_err(|err| VulkanError::SwapchainCreationError(err.to_string()))?;

        Ok(Swapchain {
            swapchain_loader,
            swapchain,
            back_buffers: back_buffer,
        })
    }
}
