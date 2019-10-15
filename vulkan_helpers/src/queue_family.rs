use ash::vk;

use crate::errors::VulkanError;
use crate::instance::Instance;
use crate::physical_device::PhysicalDevice;
use crate::surface::Surface;

pub type QueueFamily = u32;

pub struct QueueFamilyBuilder<'a> {
    instance: &'a Instance,
    surface: &'a Surface,
    physical_device: PhysicalDevice,
}

impl<'a> QueueFamilyBuilder<'a> {
    pub fn new(
        instance: &'a Instance,
        surface: &'a Surface,
        physical_device: PhysicalDevice,
    ) -> Self {
        QueueFamilyBuilder {
            instance,
            surface,
            physical_device,
        }
    }

    pub fn build(self) -> Result<QueueFamily, VulkanError> {
        let queue_family = self
            .instance
            .get_physical_device_queue_family_properties(self.physical_device)
            .into_iter()
            .enumerate()
            .find_map(|(index, queue)| {
                if queue.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                    && self
                        .surface
                        .get_physical_device_surface_support(self.physical_device, index as u32)
                {
                    Some(index as u32)
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                VulkanError::QueueFamilyCreationError(String::from("Cannot find queue family"))
            })?;

        Ok(queue_family)
    }
}
