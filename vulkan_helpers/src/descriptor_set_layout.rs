use std::rc::Rc;

use ash::vk;

use crate::device::Device;
use crate::errors::VulkanError;
use crate::vulkan_context::VulkanContext;

pub struct DescriptorSetLayout {
    device: Rc<Device>,
    descriptor_set_layout: vk::DescriptorSetLayout,
}

impl Drop for DescriptorSetLayout {
    fn drop(&mut self) {
        self.device
            .destroy_descriptor_set_layout(self.descriptor_set_layout);
    }
}

impl DescriptorSetLayout {
    pub fn get(&self) -> vk::DescriptorSetLayout {
        self.descriptor_set_layout
    }
}

pub struct DescriptorSetLayoutBuilder<'a> {
    context: &'a VulkanContext,
    texture_count: u32,
}

impl<'a> DescriptorSetLayoutBuilder<'a> {
    pub fn new(context: &'a VulkanContext) -> Self {
        DescriptorSetLayoutBuilder {
            context,
            texture_count: 0,
        }
    }

    pub fn with_texture_count(mut self, texture_count: u32) -> Self {
        self.texture_count = texture_count;
        self
    }

    pub fn build(self) -> Result<DescriptorSetLayout, VulkanError> {
        let ubo_layout_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .build();

        let ubo_mat_color_layout_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(1)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .build();

        //        let sampler_layout_binding = vk::DescriptorSetLayoutBinding::builder()
        //            .binding(2)
        //            .descriptor_count(self.texture_count)
        //            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        //            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        //            .build();

        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&[
                ubo_layout_binding,
                ubo_mat_color_layout_binding,
                //                sampler_layout_binding,
            ])
            .build();

        let descriptor_set_layout = self
            .context
            .device
            .create_descriptor_set_layout(&layout_info)?;

        Ok(DescriptorSetLayout {
            device: Rc::clone(&self.context.device),
            descriptor_set_layout,
        })
    }
}
