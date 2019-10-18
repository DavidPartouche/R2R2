use ash::vk;

use crate::errors::VulkanError;
use crate::extensions::DeviceExtensions;
use crate::instance::Instance;
use crate::surface::Surface;

struct QueueFamilyIndices {
    graphics_family: Option<usize>,
    present_family: Option<usize>,
}

impl QueueFamilyIndices {
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

pub type PhysicalDevice = vk::PhysicalDevice;

pub struct PhysicalDeviceBuilder<'a> {
    instance: &'a Instance,
    surface: &'a Surface,
    extensions: Option<&'a Vec<DeviceExtensions>>,
}

impl<'a> PhysicalDeviceBuilder<'a> {
    pub fn new(instance: &'a Instance, surface: &'a Surface) -> Self {
        PhysicalDeviceBuilder {
            instance,
            surface,
            extensions: None,
        }
    }

    pub fn with_extensions(mut self, extensions: &'a Vec<DeviceExtensions>) -> Self {
        self.extensions = Some(extensions);
        self
    }

    pub fn build(self) -> Result<PhysicalDevice, VulkanError> {
        let physical_devices = self.instance.enumerate_physical_devices()?;
        let physical_device = physical_devices
            .into_iter()
            .find(|device| self.is_device_suitable(*device))
            .ok_or_else(|| {
                VulkanError::PhysicalDeviceCreationError(String::from(
                    "Cannot find suitable physical device",
                ))
            })?;

        Ok(physical_device)
    }

    fn is_device_suitable(&self, device: vk::PhysicalDevice) -> bool {
        let indices = self.find_queue_families(device);
        let swapchain_support = self.surface.query_swapchain_support(device).unwrap();

        indices.is_complete()
            && self.check_device_extensions_support(device)
            && !swapchain_support.formats.is_empty()
            && !swapchain_support.present_modes.is_empty()
            && self
                .instance
                .get_physical_device_features(device)
                .sampler_anisotropy
                == vk::TRUE
    }

    fn find_queue_families(&self, device: vk::PhysicalDevice) -> QueueFamilyIndices {
        let queue_families = self
            .instance
            .get_physical_device_queue_family_properties(device);

        let mut graphics_family = None;
        let mut present_family = None;
        for (index, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                graphics_family = Some(index);
            }

            if self
                .surface
                .get_physical_device_surface_support(device, index as u32)
            {
                present_family = Some(index);
            }

            if graphics_family.is_some() && present_family.is_some() {
                break;
            }
        }

        QueueFamilyIndices {
            graphics_family,
            present_family,
        }
    }

    fn check_device_extensions_support(&self, device: vk::PhysicalDevice) -> bool {
        let available_extensions = self
            .instance
            .enumerate_device_extension_properties(device)
            .unwrap();

        for extension in self.extensions.unwrap_or(&vec![]) {
            if available_extensions
                .iter()
                .find(|available_extension| *available_extension == extension)
                .is_none()
            {
                return false;
            }
        }

        true
    }
}
