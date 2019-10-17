use std::rc::Rc;

use ash::vk;

use crate::buffer::Buffer;
use crate::descriptor_pool::DescriptorPool;
use crate::descriptor_set_layout::DescriptorSetLayout;
use crate::device::Device;
use crate::errors::VulkanError;
use crate::vulkan_context::VulkanContext;

pub struct DescriptorSet {
    device: Rc<Device>,
    descriptor_pool: Rc<DescriptorPool>,
    descriptor_set: vk::DescriptorSet,
}

impl Drop for DescriptorSet {
    fn drop(&mut self) {
        self.device
            .free_descriptor_sets(self.descriptor_pool.get(), &[self.descriptor_set]);
    }
}

pub struct DescriptorSetBuilder<'a> {
    context: &'a VulkanContext,
    descriptor_set_layout: &'a DescriptorSetLayout,
    uniform_buffer: &'a Buffer,
}

impl<'a> DescriptorSetBuilder<'a> {
    pub fn new(
        context: &'a VulkanContext,
        descriptor_set_layout: &'a DescriptorSetLayout,
        uniform_buffer: &'a Buffer,
    ) -> Self {
        DescriptorSetBuilder {
            context,
            descriptor_set_layout,
            uniform_buffer,
        }
    }

    pub fn build(self) -> Result<DescriptorSet, VulkanError> {
        let alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.context.descriptor_pool.get())
            .set_layouts(&[self.descriptor_set_layout.get()])
            .build();

        let descriptor_set = self.context.device.allocate_descriptor_sets(&alloc_info)?[0];

        let buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(self.uniform_buffer.get())
            .offset(0)
            .range(vk::WHOLE_SIZE)
            .build();

        let wds = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_set)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .dst_binding(0)
            .buffer_info(&[buffer_info])
            .build();

        self.context.device.update_descriptor_sets(&[wds]);

        Ok(DescriptorSet {
            device: Rc::clone(&self.context.device),
            descriptor_pool: Rc::clone(&self.context.descriptor_pool),
            descriptor_set,
        })
    }
}
