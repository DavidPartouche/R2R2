use std::os::raw::c_void;
use std::path::Path;

use vulkan_helpers::extensions::DeviceExtensions;
use vulkan_helpers::glm;
use vulkan_helpers::ray_tracing_pipeline::{RayTracingPipeline, RayTracingPipelineBuilder};
use vulkan_helpers::vulkan_context::{VulkanContext, VulkanContextBuilder};

use crate::model::Model;

pub struct Renderer {
    context: VulkanContext,
    pipeline: Option<RayTracingPipeline>,
}

impl Renderer {
    pub fn new(debug: bool, hwnd: *const c_void, width: u32, height: u32) -> Self {
        let extensions = vec![
            DeviceExtensions::ExtDescriptorIndexing,
            DeviceExtensions::KhrSwapchain,
            DeviceExtensions::NvRayTracing,
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

        Self {
            context,
            pipeline: None,
        }
    }

    pub fn load_model(&mut self, filename: &Path) {
        let mut model = Model::new(filename);

        let ray_tracing_pipeline = RayTracingPipelineBuilder::new(&self.context)
            .with_vertices(&mut model.vertices)
            .with_indices(&mut model.indices)
            .with_materials(&mut model.materials)
            .with_textures(&mut model.textures)
            .build()
            .unwrap();

        self.pipeline = Some(ray_tracing_pipeline);
    }

    pub fn set_clear_value(&mut self, clear_value: glm::Vec4) {
        self.context.set_clear_value(clear_value.into());
    }

    pub fn draw_frame(&mut self) {
        self.pipeline.as_ref().unwrap().draw();
    }
}
