use std::rc::Rc;

use ash::vk;

use crate::device::Device;
use crate::errors::VulkanError;
use crate::physical_device::PhysicalDevice;
use crate::present_mode::PresentMode;
use crate::surface::Surface;
use crate::surface_format::SurfaceFormat;
use crate::swapchain::{Swapchain, SwapchainBuilder};

pub struct SwapchainContext {
    swapchain: Swapchain,
}

impl SwapchainContext {
    pub fn get_swapchain(&self) -> vk::SwapchainKHR {
        self.swapchain.get()
    }
}

pub struct SwapchainContextBuilder<'a> {
    device: Rc<Device>,
    surface: &'a Surface,
    physical_device: PhysicalDevice,
    surface_format: SurfaceFormat,
    present_mode: PresentMode,
    old_swapchain: vk::SwapchainKHR,
    width: u32,
    height: u32,
}

impl<'a> SwapchainContextBuilder<'a> {
    pub fn new(
        device: Rc<Device>,
        surface: &'a Surface,
        physical_device: PhysicalDevice,
        surface_format: SurfaceFormat,
        present_mode: PresentMode,
    ) -> Self {
        SwapchainContextBuilder {
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

    pub fn build(self) -> Result<SwapchainContext, VulkanError> {
        let swapchain = self.create_swapchain(
            Rc::clone(&self.device),
            self.surface,
            self.physical_device,
            self.surface_format,
            self.present_mode,
        )?;
        Ok(SwapchainContext { swapchain })
    }

    fn create_swapchain(
        &self,
        device: Rc<Device>,
        surface: &Surface,
        physical_device: PhysicalDevice,
        surface_format: SurfaceFormat,
        present_mode: PresentMode,
    ) -> Result<Swapchain, VulkanError> {
        SwapchainBuilder::new(
            device,
            surface,
            physical_device,
            surface_format,
            present_mode,
        )
        .with_old_swapchain(self.old_swapchain)
        .with_width(self.width)
        .with_height(self.height)
        .build()
    }
}
