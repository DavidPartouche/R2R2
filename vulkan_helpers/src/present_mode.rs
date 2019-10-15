use ash::vk;

use crate::errors::VulkanError;
use crate::physical_device::PhysicalDevice;
use crate::surface::Surface;

pub type PresentMode = vk::PresentModeKHR;

pub struct PresentModeBuilder<'a> {
    surface: &'a Surface,
    physical_device: PhysicalDevice,
}

impl<'a> PresentModeBuilder<'a> {
    pub fn new(surface: &'a Surface, physical_device: PhysicalDevice) -> Self {
        PresentModeBuilder {
            surface,
            physical_device,
        }
    }

    pub fn build(self) -> Result<PresentMode, VulkanError> {
        let present_modes = self
            .surface
            .get_physical_device_surface_present_modes(self.physical_device)?;

        let mut result = vk::PresentModeKHR::FIFO;
        for present_mode in present_modes {
            if present_mode == vk::PresentModeKHR::MAILBOX {
                result = present_mode;
                break;
            } else if present_mode == vk::PresentModeKHR::IMMEDIATE {
                result = present_mode;
            }
        }
        Ok(result)
    }
}
