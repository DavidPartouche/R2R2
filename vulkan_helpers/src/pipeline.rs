use std::ffi::CStr;
use std::rc::Rc;

use ash::vk;

use crate::device::VulkanDevice;
use crate::errors::VulkanError;
use crate::ray_tracing::RayTracing;
use crate::ray_tracing_descriptor_set::RayTracingDescriptorSet;
use crate::shader_module::ShaderModule;
use crate::vulkan_context::VulkanContext;

pub struct Pipeline {
    device: Rc<VulkanDevice>,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        self.device.destroy_pipeline(self.pipeline);
        self.device.destroy_pipeline_layout(self.pipeline_layout);
    }
}

pub struct PipelineBuilder<'a> {
    context: &'a VulkanContext,
    ray_tracing: &'a RayTracing,
    descriptor_set: &'a RayTracingDescriptorSet,
    ray_gen_shader: Option<ShaderModule>,
    miss_shader: Option<ShaderModule>,
    closest_hit_shader: Option<ShaderModule>,
    max_recursion_depth: u32,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(
        context: &'a VulkanContext,
        ray_tracing: &'a RayTracing,
        descriptor_set: &'a RayTracingDescriptorSet,
    ) -> Self {
        PipelineBuilder {
            context,
            ray_tracing,
            descriptor_set,
            ray_gen_shader: None,
            miss_shader: None,
            closest_hit_shader: None,
            max_recursion_depth: 0,
        }
    }

    pub fn with_ray_gen_shader(mut self, ray_gen_shader: ShaderModule) -> Self {
        self.ray_gen_shader = Some(ray_gen_shader);
        self
    }

    pub fn with_miss_shader(mut self, miss_shader: ShaderModule) -> Self {
        self.miss_shader = Some(miss_shader);
        self
    }

    pub fn with_closest_hit_shader(mut self, closest_hit_shader: ShaderModule) -> Self {
        self.closest_hit_shader = Some(closest_hit_shader);
        self
    }

    pub fn with_max_recursion_depth(mut self, max_recursion_depth: u32) -> Self {
        self.max_recursion_depth = max_recursion_depth;
        self
    }

    pub fn build(self) -> Result<Pipeline, VulkanError> {
        let mut shader_stages = vec![];
        let mut shader_groups = vec![];

        let (shader_stage, shader_group) = self.add_shader_stage(
            self.ray_gen_shader.as_ref().unwrap(),
            vk::ShaderStageFlags::RAYGEN_NV,
            0,
        );
        shader_stages.push(shader_stage);
        shader_groups.push(shader_group);

        let (shader_stage, shader_group) = self.add_shader_stage(
            self.miss_shader.as_ref().unwrap(),
            vk::ShaderStageFlags::MISS_NV,
            1,
        );
        shader_stages.push(shader_stage);
        shader_groups.push(shader_group);

        let (shader_stage, shader_group) =
            self.add_closest_hit_shader(self.closest_hit_shader.as_ref().unwrap(), 2);
        shader_stages.push(shader_stage);
        shader_groups.push(shader_group);

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&[self.descriptor_set.get_layout()])
            .build();

        let pipeline_layout = self
            .context
            .device
            .create_pipeline_layout(&pipeline_layout_info)?;

        let pipeline_info = vk::RayTracingPipelineCreateInfoNV::builder()
            .stages(&shader_stages)
            .groups(&shader_groups)
            .max_recursion_depth(self.max_recursion_depth)
            .layout(pipeline_layout)
            .build();

        let pipeline = self
            .ray_tracing
            .create_ray_tracing_pipelines(&[pipeline_info])?[0];

        Ok(Pipeline {
            device: Rc::clone(&self.context.device),
            pipeline_layout,
            pipeline,
        })
    }

    fn add_shader_stage(
        &self,
        shader: &ShaderModule,
        stage: vk::ShaderStageFlags,
        index: u32,
    ) -> (
        vk::PipelineShaderStageCreateInfo,
        vk::RayTracingShaderGroupCreateInfoNV,
    ) {
        let stage_create = vk::PipelineShaderStageCreateInfo::builder()
            .stage(stage)
            .module(shader.get())
            .name(CStr::from_bytes_with_nul(b"main\0").unwrap())
            .build();
        let group_info = vk::RayTracingShaderGroupCreateInfoNV::builder()
            .ty(vk::RayTracingShaderGroupTypeNV::GENERAL)
            .general_shader(index)
            .closest_hit_shader(vk::SHADER_UNUSED_NV)
            .any_hit_shader(vk::SHADER_UNUSED_NV)
            .intersection_shader(vk::SHADER_UNUSED_NV)
            .build();
        (stage_create, group_info)
    }

    fn add_closest_hit_shader(
        &self,
        shader: &ShaderModule,
        index: u32,
    ) -> (
        vk::PipelineShaderStageCreateInfo,
        vk::RayTracingShaderGroupCreateInfoNV,
    ) {
        let stage_create = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::CLOSEST_HIT_NV)
            .module(shader.get())
            .name(CStr::from_bytes_with_nul(b"main\0").unwrap())
            .build();

        let group_info = vk::RayTracingShaderGroupCreateInfoNV::builder()
            .ty(vk::RayTracingShaderGroupTypeNV::TRIANGLES_HIT_GROUP)
            .general_shader(vk::SHADER_UNUSED_NV)
            .closest_hit_shader(index)
            .any_hit_shader(vk::SHADER_UNUSED_NV)
            .intersection_shader(vk::SHADER_UNUSED_NV)
            .build();

        (stage_create, group_info)
    }
}
