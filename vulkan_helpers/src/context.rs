use std::os::raw::c_void;
use std::ptr::null;

use crate::errors::VulkanError;
use crate::extensions::ExtensionProperties;
use crate::instance::{Instance, InstanceBuilder};
use crate::physical_device::{PhysicalDevice, PhysicalDeviceBuilder};
use crate::queue_family::{QueueFamily, QueueFamilyBuilder};
use crate::surface::{Surface, SurfaceBuilder};
use crate::surface_format::{SurfaceFormat, SurfaceFormatBuilder};

pub struct VulkanContext {
    surface_format: SurfaceFormat,
    queue_family: QueueFamily,
    physical_device: PhysicalDevice,
    surface: Surface,
    instance: Instance,
}

pub struct VulkanContextBuilder {
    debug: bool,
    hwnd: *const c_void,
    extensions: Vec<ExtensionProperties>,
}

impl Default for VulkanContextBuilder {
    fn default() -> Self {
        VulkanContextBuilder {
            debug: false,
            hwnd: null(),
            extensions: vec![],
        }
    }
}

impl VulkanContextBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_debug_enabled(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn with_hwnd(mut self, hwnd: *const c_void) -> Self {
        self.hwnd = hwnd;
        self
    }

    pub fn with_extensions(mut self, extensions: Vec<ExtensionProperties>) -> Self {
        self.extensions = extensions;
        self
    }

    pub fn build(self) -> VulkanContext {
        let instance = self.create_instance().unwrap();
        let surface = self.create_surface(&instance).unwrap();
        let physical_device = self.get_physical_device(&instance, &surface).unwrap();
        let queue_family = self
            .get_queue_family(&instance, &surface, physical_device)
            .unwrap();
        let surface_format = self.find_surface_format(&surface, physical_device).unwrap();

        VulkanContext {
            instance,
            surface,
            physical_device,
            queue_family,
            surface_format,
        }
    }

    fn create_instance(&self) -> Result<Instance, VulkanError> {
        InstanceBuilder::new()
            .with_debug_enabled(self.debug)
            .build()
    }

    fn create_surface(&self, instance: &Instance) -> Result<Surface, VulkanError> {
        SurfaceBuilder::new(instance).with_hwnd(self.hwnd).build()
    }

    fn get_physical_device(
        &self,
        instance: &Instance,
        surface: &Surface,
    ) -> Result<PhysicalDevice, VulkanError> {
        PhysicalDeviceBuilder::new(instance, surface)
            .with_extensions(&self.extensions)
            .build()
    }

    fn get_queue_family(
        &self,
        instance: &Instance,
        surface: &Surface,
        physical_device: PhysicalDevice,
    ) -> Result<QueueFamily, VulkanError> {
        QueueFamilyBuilder::new(instance, surface, physical_device).build()
    }

    fn find_surface_format(
        &self,
        surface: &Surface,
        physical_device: PhysicalDevice,
    ) -> Result<SurfaceFormat, VulkanError> {
        SurfaceFormatBuilder::new(surface, physical_device).build()
    }
}
