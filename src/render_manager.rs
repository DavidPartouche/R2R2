use std::os::raw::c_void;
use std::path::Path;
use std::ptr::null;

use vulkan_bootstrap::debug::{DebugOptions, DebugSeverity, DebugType};
use vulkan_bootstrap::extensions::DeviceExtensions;
use vulkan_bootstrap::features::Features;
use vulkan_bootstrap::vulkan_context::{VulkanContext, VulkanContextBuilder};
use vulkan_bootstrap::windows::Win32Window;

use vulkan_ray_tracing::geometry_instance::GeometryInstanceBuilder;
use vulkan_ray_tracing::glm;
use vulkan_ray_tracing::ray_tracing_pipeline::{RayTracingPipeline, RayTracingPipelineBuilder};

use crate::model::Model;

pub struct RenderManager {
    context: VulkanContext,
    pipeline: Option<RayTracingPipeline>,
    width: u32,
    height: u32,
}

impl RenderManager {
    pub fn new(debug: bool, hwnd: *const c_void, width: u32, height: u32) -> Self {
        let extensions = vec![
            DeviceExtensions::ExtDescriptorIndexing,
            DeviceExtensions::KhrSwapchain,
            DeviceExtensions::NvRayTracing,
        ];

        let debug_options = if debug {
            DebugOptions {
                debug_severity: DebugSeverity {
                    warning: true,
                    error: true,
                    info: false,
                    verbose: false,
                },
                debug_type: DebugType::all(),
            }
        } else {
            DebugOptions {
                debug_severity: DebugSeverity::none(),
                debug_type: DebugType::none(),
            }
        };

        let window = Win32Window {
            hinstance: null(),
            hwnd,
            width,
            height,
        };

        let context = VulkanContextBuilder::new()
            .with_debug_options(debug_options)
            .with_window(window)
            .with_extensions(extensions)
            .with_features(Features::all())
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

    pub fn set_clear_color(&mut self, clear_color: &glm::Vec4) {
        self.context.set_clear_value((*clear_color).into());
    }

    pub fn load_model(&mut self, filename: &Path) {
        let mut model = Model::new(filename);

        let geom = GeometryInstanceBuilder::new(&self.context)
            .with_vertices(&mut model.vertices)
            .with_indices(&mut model.indices)
            .with_materials(&mut model.materials)
            .with_textures(&mut model.textures)
            .build()
            .unwrap();

        let ray_tracing_pipeline = RayTracingPipelineBuilder::new(&self.context)
            .with_geometry_instance(geom)
            .build()
            .unwrap();

        self.pipeline = Some(ray_tracing_pipeline);
    }

    pub fn draw(&mut self) {
        let pipeline = self.pipeline.as_mut().expect("No scene loaded.");

        pipeline
            .update_camera_buffer(self.width as f32, self.height as f32)
            .unwrap();

        pipeline.draw(&mut self.context).unwrap();
    }
}
