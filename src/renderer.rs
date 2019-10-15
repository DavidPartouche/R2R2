use std::os::raw::c_void;
use vulkan_helpers::context::{VulkanContext, VulkanContextBuilder};

pub struct Renderer {
    context: VulkanContext,
}

impl Renderer {
    pub fn new(debug: bool, hwnd: *const c_void) -> Self {
        let context = VulkanContextBuilder::new()
            .with_debug_enabled(debug)
            .with_hwnd(hwnd)
            .build();
        Self { context }
    }

    pub fn draw_frame(&self) {
        //        println!("Window running");
    }
}
