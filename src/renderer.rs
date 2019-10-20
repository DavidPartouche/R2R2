use std::mem;
use std::os::raw::c_void;
use std::path::Path;

use vulkan_helpers::extensions::DeviceExtensions;
use vulkan_helpers::glm;
use vulkan_helpers::pipeline_context::{GraphicsPipelineContext, GraphicsPipelineContextBuilder};
use vulkan_helpers::ray_tracing_pipeline::RayTracingPipelineBuilder;
use vulkan_helpers::vulkan_context::{VulkanContext, VulkanContextBuilder};

use crate::model::Model;

pub struct Renderer {
    context: VulkanContext,
    //    models: Vec<GeometryInstance>,
    graphics_pipeline: Option<GraphicsPipelineContext>,
    width: f32,
    height: f32,
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
            //            models: vec![],
            graphics_pipeline: None,
            width: width as f32,
            height: height as f32,
        }
    }

    pub fn load_model(&mut self, filename: &Path) {
        //        let model = Model::new(filename);

        //        let graphics_pipeline = GraphicsPipelineContextBuilder::new(&self.context)
        //            .with_vertex_shader(Path::new("assets/shaders/vert_shader.spv"))
        //            .with_fragment_shader(Path::new("assets/shaders/frag_shader.spv"))
        //            .with_ubo_size(mem::size_of::<UniformBufferObject>())
        //            .with_material_buffer(material_buffer)
        //            .with_textures(textures)
        //            .with_indices_count(model.indices.len())
        //            .build()
        //            .unwrap();

        //        self.graphics_pipeline = Some(graphics_pipeline);

        let mut model = Model::new(filename);

        let ray_tracing_pipeline = RayTracingPipelineBuilder::new(&self.context)
            .with_vertices(&mut model.vertices)
            .with_indices(&mut model.indices)
            .with_materials(&mut model.materials)
            .with_textures(&mut model.textures)
            .build();
    }

    pub fn set_clear_value(&mut self, clear_value: glm::Vec4) {
        self.context.set_clear_value(clear_value.into());
    }

    pub fn draw_frame(&mut self) {
        //        self.update_uniform_buffer();

        //        let vertex_buffer = self.models[0].vertex_buffer.get();
        //        let index_buffer = self.models[0].index_buffer.get();
        //        let graphics_pipeline = self.graphics_pipeline.as_ref().unwrap();
        //        let command_buffer = self.context.get_current_command_buffer();

        //        self.context.frame_begin().unwrap();

        //        self.context.begin_render_pass();
        //        graphics_pipeline.draw(command_buffer, vertex_buffer, index_buffer);
        //        self.context.end_render_pass();

        //        self.context.frame_end().unwrap();
        //        self.context.frame_present().unwrap();
    }

    //    fn update_uniform_buffer(&self) {
    //        let aspect_ratio = self.width / self.height;
    //        let model = glm::identity();
    //        let model_it = glm::inverse_transpose(model);
    //        let view = glm::look_at(
    //            &glm::vec3(1.0, 1.0, 1.0),
    //            &glm::vec3(0.0, 0.0, 0.0),
    //            &glm::vec3(0.0, 1.0, 0.0),
    //        );
    //        let mut proj = glm::perspective(f32::to_radians(65.0), aspect_ratio, 0.1, 1000.0);
    //        proj[(1, 1)] = -1.0;
    //        let ubo = UniformBufferObject {
    //            model,
    //            view,
    //            proj,
    //            model_it,
    //        };
    //
    //        let ubo = &ubo as *const UniformBufferObject as *const c_void;
    //        self.graphics_pipeline
    //            .as_ref()
    //            .unwrap()
    //            .update_uniform_buffer(ubo)
    //            .unwrap();
    //    }
}
