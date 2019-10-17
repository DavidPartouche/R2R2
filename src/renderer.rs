use std::mem;
use std::os::raw::c_void;
use std::path::Path;

use nalgebra_glm as glm;

use vulkan_helpers::extensions::ExtensionProperties;
use vulkan_helpers::pipeline_context::{GraphicsPipelineContext, GraphicsPipelineContextBuilder};
use vulkan_helpers::vulkan_context::{VulkanContext, VulkanContextBuilder};

#[repr(C, packed)]
struct UniformBufferObject {
    model: glm::Mat4,
    view: glm::Mat4,
    proj: glm::Mat4,
}

pub struct Renderer {
    graphics_pipeline: GraphicsPipelineContext,
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
            .with_frames_count(2)
            .build()
            .unwrap();

        let graphics_pipeline = GraphicsPipelineContextBuilder::new(&context)
            .with_texture_count(0)
            .with_vertex_shader(Path::new("assets/shaders/vert_shader.spv"))
            .with_fragment_shader(Path::new("assets/shaders/frag_shader.spv"))
            .with_ubo_size(mem::size_of::<UniformBufferObject>())
            .build();

        Self {
            context,
            graphics_pipeline,
        }
    }

    pub fn set_clear_value(&mut self, clear_value: glm::Vec4) {
        self.context.set_clear_value(clear_value.into());
    }

    pub fn draw_frame(&mut self) {
        self.context.frame_begin().unwrap();

        // Render
        self.context.begin_render_pass();
        self.context.draw(self.graphics_pipeline.get());
        self.context.end_render_pass();

        self.context.frame_end().unwrap();
        self.context.frame_present().unwrap();
    }
}
