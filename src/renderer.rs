use std::os::raw::c_void;

use vulkan_helpers::context::{VulkanContext, VulkanContextBuilder};
use vulkan_helpers::extensions::ExtensionProperties;

pub struct Renderer {
    context: VulkanContext,
}

impl Renderer {
    pub fn new(debug: bool, hwnd: *const c_void, width: u32, height: u32) -> Self {
        let extensions = vec![
            ExtensionProperties::KhrSwapchain,
            ExtensionProperties::NvRayTracing,
        ];
        let context = VulkanContextBuilder::new()
            .with_debug_enabled(debug)
            .with_hwnd(hwnd)
            .with_width(width)
            .with_height(height)
            .with_extensions(extensions)
            .build()
            .unwrap();
        Self { context }
    }

    pub fn draw_frame(&self) {
        //        println!("Window running");
    }
}
