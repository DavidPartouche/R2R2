use std::ptr::null;

use ash::extensions::khr;
use ash::vk;

use crate::errors::VulkanError;
use crate::instance::Instance;

pub struct SwapchainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

pub struct Surface {
    surface_loader: khr::Surface,
    surface: vk::SurfaceKHR,
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
        }
    }
}

impl Surface {
    pub fn get_physical_device_surface_support(
        &self,
        device: vk::PhysicalDevice,
        index: u32,
    ) -> bool {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_support(device, index, self.surface)
        }
    }

    pub fn query_swapchain_support(
        &self,
        device: vk::PhysicalDevice,
    ) -> Result<SwapchainSupportDetails, VulkanError> {
        let capabilities = unsafe {
            self.surface_loader
                .get_physical_device_surface_capabilities(device, self.surface)
        }
        .map_err(|err| VulkanError::SurfaceError(err.to_string()))?;

        let formats = unsafe {
            self.surface_loader
                .get_physical_device_surface_formats(device, self.surface)
        }
        .map_err(|err| VulkanError::SurfaceError(err.to_string()))?;

        let present_modes = unsafe {
            self.surface_loader
                .get_physical_device_surface_present_modes(device, self.surface)
        }
        .map_err(|err| VulkanError::SurfaceError(err.to_string()))?;

        Ok(SwapchainSupportDetails {
            capabilities,
            formats,
            present_modes,
        })
    }

    pub fn get_physical_device_surface_formats(
        &self,
        device: vk::PhysicalDevice,
    ) -> Result<Vec<vk::SurfaceFormatKHR>, VulkanError> {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_formats(device, self.surface)
        }
        .map_err(|err| VulkanError::SurfaceError(err.to_string()))
    }
}

pub struct SurfaceBuilder<'a> {
    instance: &'a Instance,
    hwnd: vk::HWND,
}

impl<'a> SurfaceBuilder<'a> {
    pub fn new(instance: &'a Instance) -> Self {
        SurfaceBuilder {
            instance,
            hwnd: null(),
        }
    }

    pub fn with_hwnd(mut self, hwnd: vk::HWND) -> Self {
        self.hwnd = hwnd;
        self
    }

    pub fn build(self) -> Result<Surface, VulkanError> {
        let (surface_loader, surface) = self.instance.create_win_32_surface(self.hwnd)?;

        Ok(Surface {
            surface_loader,
            surface,
        })
    }
}
