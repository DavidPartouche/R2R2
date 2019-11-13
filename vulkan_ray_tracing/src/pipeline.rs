use std::ffi::CStr;
use std::rc::Rc;

use ash::vk;
use vulkan_bootstrap::device::VulkanDevice;
use vulkan_bootstrap::errors::VulkanError;
use vulkan_bootstrap::shader_module::ShaderModule;
use vulkan_bootstrap::vulkan_context::VulkanContext;

use crate::descriptor_set::DescriptorSet;
use crate::ray_tracing::RayTracing;

pub struct Pipeline {
    device: Rc<VulkanDevice>,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    pub ray_gen_index: u32,
    pub miss_index: u32,
    pub shadow_miss_index: u32,
    pub hit_group_index: u32,
    pub shadow_hit_group_index: u32,
}

impl Pipeline {
    pub fn get(&self) -> vk::Pipeline {
        self.pipeline
    }

    pub fn get_layout(&self) -> vk::PipelineLayout {
        self.pipeline_layout
    }
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
    descriptor_set: &'a DescriptorSet,
    ray_gen_shader: Option<ShaderModule>,
    miss_shader: Option<ShaderModule>,
    shadow_miss_shader: Option<ShaderModule>,
    hit_shader: Option<ShaderModule>,
    max_recursion_depth: u32,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(
        context: &'a VulkanContext,
        ray_tracing: &'a RayTracing,
        descriptor_set: &'a DescriptorSet,
    ) -> Self {
        PipelineBuilder {
            context,
            ray_tracing,
            descriptor_set,
            ray_gen_shader: None,
            miss_shader: None,
            shadow_miss_shader: None,
            hit_shader: None,
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

    pub fn with_shadow_miss_shader(mut self, shadow_miss_shader: ShaderModule) -> Self {
        self.shadow_miss_shader = Some(shadow_miss_shader);
        self
    }

    pub fn with_hit_shader(mut self, hit_shader: ShaderModule) -> Self {
        self.hit_shader = Some(hit_shader);
        self
    }

    pub fn with_max_recursion_depth(mut self, max_recursion_depth: u32) -> Self {
        self.max_recursion_depth = max_recursion_depth;
        self
    }

    pub fn build(self) -> Result<Pipeline, VulkanError> {
        let mut shader_stages = vec![];
        let mut shader_groups = vec![];

        let ray_gen_index = self.add_shader_stage(
            self.ray_gen_shader.as_ref(),
            vk::ShaderStageFlags::RAYGEN_NV,
            &mut shader_stages,
            &mut shader_groups,
        );

        let miss_index = self.add_shader_stage(
            self.miss_shader.as_ref(),
            vk::ShaderStageFlags::MISS_NV,
            &mut shader_stages,
            &mut shader_groups,
        );

        let shadow_miss_index = self.add_shader_stage(
            self.shadow_miss_shader.as_ref(),
            vk::ShaderStageFlags::MISS_NV,
            &mut shader_stages,
            &mut shader_groups,
        );

        let hit_group_index = self.add_shader_stage(
            self.hit_shader.as_ref(),
            vk::ShaderStageFlags::CLOSEST_HIT_NV,
            &mut shader_stages,
            &mut shader_groups,
        );

        let shadow_hit_group_index = self.add_shader_stage(
            None,
            vk::ShaderStageFlags::empty(),
            &mut shader_stages,
            &mut shader_groups,
        );

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&[self.descriptor_set.get_layout()])
            .build();

        let pipeline_layout = self
            .context
            .get_device()
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
            device: Rc::clone(&self.context.get_device()),
            pipeline_layout,
            pipeline,
            ray_gen_index,
            miss_index,
            shadow_miss_index,
            hit_group_index,
            shadow_hit_group_index,
        })
    }

    fn add_shader_stage(
        &self,
        shader: Option<&ShaderModule>,
        stage: vk::ShaderStageFlags,
        shader_stages: &mut Vec<vk::PipelineShaderStageCreateInfo>,
        shader_groups: &mut Vec<vk::RayTracingShaderGroupCreateInfoNV>,
    ) -> u32 {
        let index = shader_stages.len() as u32;

        let mut group_info = vk::RayTracingShaderGroupCreateInfoNV::builder()
            .ty(vk::RayTracingShaderGroupTypeNV::TRIANGLES_HIT_GROUP)
            .general_shader(vk::SHADER_UNUSED_NV)
            .closest_hit_shader(vk::SHADER_UNUSED_NV)
            .any_hit_shader(vk::SHADER_UNUSED_NV)
            .intersection_shader(vk::SHADER_UNUSED_NV);

        if let Some(shader) = shader {
            let stage_create = vk::PipelineShaderStageCreateInfo::builder()
                .stage(stage)
                .module(shader.get())
                .name(CStr::from_bytes_with_nul(b"main\0").unwrap())
                .build();
            shader_stages.push(stage_create);

            match stage {
                vk::ShaderStageFlags::ANY_HIT_NV => {
                    group_info = group_info.any_hit_shader(index);
                }
                vk::ShaderStageFlags::CLOSEST_HIT_NV => {
                    group_info = group_info.closest_hit_shader(index);
                }
                vk::ShaderStageFlags::INTERSECTION_NV => {
                    group_info = group_info.intersection_shader(index);
                }
                _ => {
                    group_info = group_info
                        .ty(vk::RayTracingShaderGroupTypeNV::GENERAL)
                        .general_shader(index);
                }
            }
        }

        let group_info = group_info.build();
        shader_groups.push(group_info);

        index
    }
}
