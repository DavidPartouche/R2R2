use std::rc::Rc;

use ash::extensions::khr;
use ash::vk;

use crate::device::Device;
use crate::errors::VulkanError;
use crate::physical_device::PhysicalDevice;
use crate::present_mode::PresentMode;
use crate::surface::Surface;
use crate::surface_format::SurfaceFormat;

pub struct Swapchain {
    device: Rc<Device>,
    swapchain_loader: khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    back_buffer: Vec<vk::Image>,
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
    pub fn get(&self) -> vk::SwapchainKHR {
        self.swapchain
    }
}

pub struct SwapchainBuilder<'a> {
    device: Rc<Device>,
    surface: &'a Surface,
    physical_device: PhysicalDevice,
    surface_format: SurfaceFormat,
    present_mode: PresentMode,
    old_swapchain: vk::SwapchainKHR,
    width: u32,
    height: u32,
}

impl<'a> SwapchainBuilder<'a> {
    pub fn new(
        device: Rc<Device>,
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
            old_swapchain: vk::SwapchainKHR::null(),
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

    pub fn with_old_swapchain(mut self, swapchain: vk::SwapchainKHR) -> Self {
        self.old_swapchain = swapchain;
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
            .old_swapchain(self.old_swapchain)
            .min_image_count(image_count)
            .image_extent(vk::Extent2D::builder().width(width).height(height).build())
            .build();

        let swapchain_loader = self.device.new_swapchain();
        let swapchain = unsafe { swapchain_loader.create_swapchain(&info, None) }
            .map_err(|err| VulkanError::SwapchainCreationError(err.to_string()))?;

        let back_buffer = unsafe { swapchain_loader.get_swapchain_images(swapchain) }
            .map_err(|err| VulkanError::SwapchainCreationError(err.to_string()))?;

        Ok(Swapchain {
            device: self.device,
            swapchain_loader,
            swapchain,
            back_buffer,
        })
    }
}
