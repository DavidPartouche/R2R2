use crate::errors::VulkanError;
use crate::instance::VulkanInstance;
use ash::extensions::khr;
use ash::vk;
use std::ptr::null;

pub struct VulkanSurface {
    surface_loader: khr::Surface,
    surface: vk::SurfaceKHR,
}

impl Drop for VulkanSurface {
    fn drop(&mut self) {
        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
        }
    }
}

pub struct VulkanSurfaceBuilder<'a> {
    instance: &'a VulkanInstance,
    hwnd: vk::HWND,
}

impl<'a> VulkanSurfaceBuilder<'a> {
    pub fn new(instance: &'a VulkanInstance) -> Self {
        VulkanSurfaceBuilder {
            instance,
            hwnd: null(),
        }
    }

    pub fn with_hwnd(mut self, hwnd: vk::HWND) -> Self {
        self.hwnd = hwnd;
        self
    }

    pub fn build(self) -> Result<VulkanSurface, VulkanError> {
        let (surface_loader, surface) = self.instance.create_win_32_surface(self.hwnd)?;

        Ok(VulkanSurface {
            surface_loader,
            surface,
        })
    }
}
