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
    width: u32,
    height: u32,
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
            width,
            height,
        }
    }

    pub fn set_clear_color(&mut self, clear_color: glm::Vec4) {
        self.context.set_clear_value(clear_color);
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

    pub fn draw(&mut self) {
        let pipeline = self.pipeline.as_mut().unwrap();

        pipeline
            .update_camera_buffer(self.width as f32, self.height as f32)
            .unwrap();

        pipeline.draw(&mut self.context).unwrap();
    }
}
