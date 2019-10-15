use crate::errors::VulkanError;
use crate::instance::{VulkanInstance, VulkanInstanceBuilder};
use crate::surface::{VulkanSurface, VulkanSurfaceBuilder};
use std::os::raw::c_void;
use std::ptr::null;

pub struct VulkanContext {
    surface: VulkanSurface,
    instance: VulkanInstance,
}

pub struct VulkanContextBuilder {
    debug: bool,
    hwnd: *const c_void,
}

impl VulkanContextBuilder {
    pub fn new() -> Self {
        VulkanContextBuilder {
            debug: false,
            hwnd: null(),
        }
    }

    pub fn with_debug_enabled(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn with_hwnd(mut self, hwnd: *const c_void) -> Self {
        self.hwnd = hwnd;
        self
    }

    pub fn build(self) -> VulkanContext {
        let instance = self.create_instance().unwrap();
        let surface = self.create_surface(&instance).unwrap();

        VulkanContext { instance, surface }
    }

    fn create_instance(&self) -> Result<VulkanInstance, VulkanError> {
        VulkanInstanceBuilder::new()
            .with_debug_enabled(self.debug)
            .build()
    }

    fn create_surface(&self, instance: &VulkanInstance) -> Result<VulkanSurface, VulkanError> {
        VulkanSurfaceBuilder::new(instance)
            .with_hwnd(self.hwnd)
            .build()
    }
}
