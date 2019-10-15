use std::rc::Rc;

use ash::vk;

use crate::device::Device;
use crate::errors::VulkanError;

pub struct DescriptorPool {
    device: Rc<Device>,
    descriptor_pool: vk::DescriptorPool,
}

impl Drop for DescriptorPool {
    fn drop(&mut self) {
        self.device.destroy_descriptor_pool(self.descriptor_pool);
    }
}

pub struct DescriptorPoolBuilder {
    device: Rc<Device>,
}

impl DescriptorPoolBuilder {
    pub fn new(device: Rc<Device>) -> Self {
        DescriptorPoolBuilder { device }
    }

    pub fn build(self) -> Result<DescriptorPool, VulkanError> {
        let sampler_pool_size = vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1000)
            .build();
        let uniform_pool_size = vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1000)
            .build();
        let storage_pool_size = vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1000)
            .build();

        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET)
            .max_sets(1000)
            .pool_sizes(&[sampler_pool_size, uniform_pool_size, storage_pool_size])
            .build();

        let descriptor_pool = self.device.create_descriptor_pool(&pool_info)?;

        Ok(DescriptorPool {
            device: self.device,
            descriptor_pool,
        })
    }
}
