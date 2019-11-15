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

use crate::camera_manager::CameraManager;
use crate::model::Model;
use std::rc::Rc;

pub struct RenderManager {
    context: VulkanContext,
    camera_manager: Rc<CameraManager>,
    pipeline: Option<RayTracingPipeline>,
}

impl RenderManager {
    pub fn new(
        debug: bool,
        hwnd: *const c_void,
        width: u32,
        height: u32,
        camera_manager: Rc<CameraManager>,
    ) -> Self {
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
            camera_manager,
            pipeline: None,
        }
    }

    pub fn set_clear_color(&mut self, clear_color: glm::Vec4) {
        self.context.set_clear_value(clear_color.into());
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
            .with_camera_buffer_size(self.camera_manager.get_camera_buffer_size() as u64)
            .build()
            .unwrap();

        self.pipeline = Some(ray_tracing_pipeline);
    }

    pub fn render_scene(&mut self) {
        let pipeline = self.pipeline.as_mut().unwrap();
        pipeline
            .update_camera_buffer(self.camera_manager.get_camera_buffer(), &self.context)
            .unwrap();

        pipeline.begin_draw(&mut self.context).unwrap();
        pipeline.draw(&self.context).unwrap();
        pipeline.end_draw(&mut self.context).unwrap();
    }
}
