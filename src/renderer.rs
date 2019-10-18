use std::mem;
use std::os::raw::c_void;
use std::path::Path;

use nalgebra_glm as glm;

use vulkan_helpers::extensions::DeviceExtensions;
use vulkan_helpers::pipeline_context::{GraphicsPipelineContext, GraphicsPipelineContextBuilder};
use vulkan_helpers::vulkan_context::{VulkanContext, VulkanContextBuilder};

use crate::model::Model;

#[repr(C, packed)]
struct UniformBufferObject {
    model: glm::Mat4,
    view: glm::Mat4,
    proj: glm::Mat4,
    model_it: glm::Mat4,
}

pub struct Renderer {
    context: VulkanContext,
    graphics_pipeline: Option<GraphicsPipelineContext>,
    width: f32,
    height: f32,
}

impl Renderer {
    pub fn new(debug: bool, hwnd: *const c_void, width: u32, height: u32) -> Self {
        let extensions = vec![
            DeviceExtensions::ExtDescriptorIndexing,
            DeviceExtensions::KhrMaintenance3,
            DeviceExtensions::KhrSwapchain,
            //            ExtensionProperties::NvRayTracing,
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
            graphics_pipeline: None,
            width: width as f32,
            height: height as f32,
        }
    }

    pub fn load_model(&mut self, filename: &Path) {
        let model = Model::new(filename);
        let vertex_buffer = self.context.create_vertex_buffer(&model.vertices).unwrap();
        let index_buffer = self.context.create_index_buffer(&model.indices).unwrap();
        let material_buffer = self
            .context
            .create_material_buffer(&model.materials)
            .unwrap();
        let textures = self.context.create_texture_images(&model.textures).unwrap();

        let graphics_pipeline = GraphicsPipelineContextBuilder::new(&self.context)
            .with_vertex_shader(Path::new("assets/shaders/vert_shader.spv"))
            .with_fragment_shader(Path::new("assets/shaders/frag_shader.spv"))
            .with_ubo_size(mem::size_of::<UniformBufferObject>())
            .with_vertex_buffer(vertex_buffer)
            .with_index_buffer(index_buffer)
            .with_material_buffer(material_buffer)
            .with_textures(textures)
            .with_indices_count(model.indices.len())
            .build()
            .unwrap();

        self.graphics_pipeline = Some(graphics_pipeline);
    }

    pub fn set_clear_value(&mut self, clear_value: glm::Vec4) {
        self.context.set_clear_value(clear_value.into());
    }

    pub fn draw_frame(&mut self) {
        self.update_uniform_buffer();

        self.context.frame_begin().unwrap();

        self.context.begin_render_pass();
        self.graphics_pipeline
            .as_ref()
            .unwrap()
            .draw(self.context.get_current_command_buffer());
        self.context.end_render_pass();

        self.context.frame_end().unwrap();
        self.context.frame_present().unwrap();
    }

    fn update_uniform_buffer(&self) {
        let aspect_ratio = self.width / self.height;
        let model = glm::identity();
        let model_it = glm::inverse_transpose(model);
        let view = glm::look_at(
            &glm::vec3(1.0, 1.0, 1.0),
            &glm::vec3(0.0, 0.0, 0.0),
            &glm::vec3(0.0, 1.0, 0.0),
        );
        let mut proj = glm::perspective(f32::to_radians(65.0), aspect_ratio, 0.1, 1000.0);
        proj[(1, 1)] = -1.0;
        let ubo = UniformBufferObject {
            model,
            view,
            proj,
            model_it,
        };

        let ubo = &ubo as *const UniformBufferObject as *const c_void;
        self.graphics_pipeline
            .as_ref()
            .unwrap()
            .update_uniform_buffer(ubo)
            .unwrap();
    }
}
