use std::ffi::CStr;
use std::path::Path;
use std::rc::Rc;

use ash::vk;

use crate::descriptor_set_layout::DescriptorSetLayout;
use crate::device::VulkanDevice;
use crate::errors::VulkanError;
use crate::shader_module::ShaderModuleBuilder;
use crate::vertex::Vertex;
use crate::vulkan_context::VulkanContext;

pub struct Pipeline {
    device: Rc<VulkanDevice>,
    pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        self.device.destroy_pipeline(self.graphics_pipeline);
        self.device.destroy_pipeline_layout(self.pipeline_layout);
    }
}

impl Pipeline {
    pub fn get(&self) -> vk::Pipeline {
        self.graphics_pipeline
    }
    pub fn get_layout(&self) -> vk::PipelineLayout {
        self.pipeline_layout
    }
}

pub struct PipelineBuilder<'a> {
    context: &'a VulkanContext,
    descriptor_set_layout: &'a DescriptorSetLayout,
    vertex_shader: Option<&'a Path>,
    fragment_shader: Option<&'a Path>,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(context: &'a VulkanContext, descriptor_set_layout: &'a DescriptorSetLayout) -> Self {
        PipelineBuilder {
            context,
            descriptor_set_layout,
            vertex_shader: None,
            fragment_shader: None,
        }
    }

    pub fn with_vertex_shader(mut self, vertex_shader: &'a Path) -> Self {
        self.vertex_shader = Some(vertex_shader);
        self
    }

    pub fn with_fragment_shader(mut self, fragment_shader: &'a Path) -> Self {
        self.fragment_shader = Some(fragment_shader);
        self
    }

    pub fn build(self) -> Result<Pipeline, VulkanError> {
        let vert_shader_module = ShaderModuleBuilder::new(Rc::clone(&self.context.device))
            .with_path(&self.vertex_shader.expect("Vertex shader not specified"))
            .build()?;
        let frag_shader_module = ShaderModuleBuilder::new(Rc::clone(&self.context.device))
            .with_path(&self.fragment_shader.expect("Fragment shader not specified"))
            .build()?;

        let vert_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module.get())
            .name(CStr::from_bytes_with_nul(b"main\0").unwrap())
            .build();
        let frag_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module.get())
            .name(CStr::from_bytes_with_nul(b"main\0").unwrap())
            .build();

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&[Vertex::get_binding_description()])
            .vertex_attribute_descriptions(&Vertex::get_attribute_descriptions())
            .build();

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false)
            .build();

        let viewport = vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(self.context.width as f32)
            .height(self.context.height as f32)
            .min_depth(0.0)
            .max_depth(1.0)
            .build();

        let scissor = vk::Rect2D::builder()
            .offset(vk::Offset2D::builder().x(0).y(0).build())
            .extent(
                vk::Extent2D::builder()
                    .width(self.context.width)
                    .height(self.context.height)
                    .build(),
            )
            .build();

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .viewports(&[viewport])
            .scissor_count(1)
            .scissors(&[scissor])
            .build();

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false)
            .build();

        let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .build();

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::all())
            .blend_enable(false)
            .build();

        let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&[color_blend_attachment])
            .blend_constants([0.0, 0.0, 0.0, 0.0])
            .build();

        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false)
            .build();

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&[self.descriptor_set_layout.get()])
            .build();

        let pipeline_layout = self
            .context
            .device
            .create_pipeline_layout(&pipeline_layout_info)?;

        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&[vert_stage_info, frag_stage_info])
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .depth_stencil_state(&depth_stencil)
            .layout(pipeline_layout)
            .render_pass(self.context.render_pass.get())
            .subpass(0)
            .build();

        let graphics_pipeline = self
            .context
            .device
            .create_graphics_pipelines(&[pipeline_info])?[0];

        Ok(Pipeline {
            device: Rc::clone(&self.context.device),
            pipeline_layout,
            graphics_pipeline,
        })
    }
}
