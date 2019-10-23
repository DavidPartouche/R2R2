use ash::vk;

use crate::errors::VulkanError;
use crate::physical_device::PhysicalDevice;
use crate::surface::Surface;

pub type SurfaceFormat = vk::SurfaceFormatKHR;

pub struct SurfaceFormatBuilder<'a> {
    surface: &'a Surface,
    physical_device: &'a PhysicalDevice,
}

impl<'a> SurfaceFormatBuilder<'a> {
    pub fn new(surface: &'a Surface, physical_device: &'a PhysicalDevice) -> Self {
        SurfaceFormatBuilder {
            surface,
            physical_device,
        }
    }

    pub fn build(self) -> Result<SurfaceFormat, VulkanError> {
        let formats = self
            .surface
            .get_physical_device_surface_formats(self.physical_device.get())?;

        let format = if formats.len() == 1 {
            if formats[0].format == vk::Format::UNDEFINED {
                vk::SurfaceFormatKHR::builder()
                    .format(vk::Format::B8G8R8A8_UNORM)
                    .color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
                    .build()
            } else {
                formats[0]
            }
        } else {
            let request_formats = vec![
                vk::Format::B8G8R8A8_UNORM,
                vk::Format::R8G8B8A8_UNORM,
                vk::Format::B8G8R8_UNORM,
                vk::Format::R8G8B8_UNORM,
            ];
            let request_color_space = vk::ColorSpaceKHR::SRGB_NONLINEAR;
            let mut found = None;
            for request_format in request_formats {
                found = formats.iter().find(|format| {
                    format.format == request_format && format.color_space == request_color_space
                });
                if found.is_some() {
                    break;
                }
            }
            *found.unwrap_or(&formats[0])
        };

        Ok(format)
    }
}
