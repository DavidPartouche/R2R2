use std::os::raw::c_char;
use std::rc::Rc;

use ash::extensions::khr;
use ash::version::DeviceV1_0;
use ash::vk;

use crate::errors::VulkanError;
use crate::extensions::ExtensionProperties;
use crate::instance::Instance;
use crate::physical_device::PhysicalDevice;
use crate::queue_family::QueueFamily;

pub struct Device {
    instance: Rc<Instance>,
    device: ash::Device,
    queue: vk::Queue,
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}

impl Device {
    pub fn queue_wait_idle(&self) -> Result<(), VulkanError> {
        unsafe { self.device.queue_wait_idle(self.queue) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))?;

        Ok(())
    }

    pub fn create_command_pool(
        &self,
        pool_info: &vk::CommandPoolCreateInfo,
    ) -> Result<vk::CommandPool, VulkanError> {
        unsafe { self.device.create_command_pool(pool_info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_command_pool(&self, command_pool: vk::CommandPool) {
        unsafe {
            self.device.destroy_command_pool(command_pool, None);
        }
    }

    pub fn allocate_command_buffers(
        &self,
        alloc_info: &vk::CommandBufferAllocateInfo,
    ) -> Result<Vec<vk::CommandBuffer>, VulkanError> {
        unsafe { self.device.allocate_command_buffers(&alloc_info) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn free_command_buffers(
        &self,
        command_pool: vk::CommandPool,
        command_buffers: &[vk::CommandBuffer],
    ) {
        unsafe {
            self.device
                .free_command_buffers(command_pool, command_buffers);
        }
    }

    pub fn create_fence(&self, fence_info: &vk::FenceCreateInfo) -> Result<vk::Fence, VulkanError> {
        unsafe { self.device.create_fence(&fence_info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_fence(&self, fence: vk::Fence) {
        unsafe {
            self.device.destroy_fence(fence, None);
        }
    }

    pub fn create_semaphore(
        &self,
        semaphore_info: &vk::SemaphoreCreateInfo,
    ) -> Result<vk::Semaphore, VulkanError> {
        unsafe { self.device.create_semaphore(&semaphore_info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_semaphore(&self, semaphore: vk::Semaphore) {
        unsafe {
            self.device.destroy_semaphore(semaphore, None);
        }
    }

    pub fn create_descriptor_pool(
        &self,
        pool_info: &vk::DescriptorPoolCreateInfo,
    ) -> Result<vk::DescriptorPool, VulkanError> {
        unsafe { self.device.create_descriptor_pool(&pool_info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_descriptor_pool(&self, descriptor_pool: vk::DescriptorPool) {
        unsafe {
            self.device.destroy_descriptor_pool(descriptor_pool, None);
        }
    }

    pub fn new_swapchain(&self) -> khr::Swapchain {
        khr::Swapchain::new(self.instance.get(), &self.device)
    }
}

pub struct DeviceBuilder<'a> {
    instance: Rc<Instance>,
    physical_device: PhysicalDevice,
    queue_family: QueueFamily,
    extensions: Option<&'a Vec<ExtensionProperties>>,
}

impl<'a> DeviceBuilder<'a> {
    pub fn new(
        instance: Rc<Instance>,
        physical_device: PhysicalDevice,
        queue_family: QueueFamily,
    ) -> Self {
        DeviceBuilder {
            instance,
            physical_device,
            queue_family,
            extensions: None,
        }
    }

    pub fn with_extensions(mut self, extensions: &'a Vec<ExtensionProperties>) -> Self {
        self.extensions = Some(extensions);
        self
    }

    pub fn build(self) -> Result<Device, VulkanError> {
        let queue_priority = [1.];

        let queue_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(self.queue_family)
            .queue_priorities(&queue_priority)
            .build();

        let extension_names: Vec<*const c_char> = self
            .extensions
            .unwrap_or(&vec![])
            .iter()
            .map(|extension| extension.name().as_ptr())
            .collect();

        let mut desc_index_features =
            vk::PhysicalDeviceDescriptorIndexingFeaturesEXT::builder().build();

        let supported_features = self
            .instance
            .get_physical_device_features2(self.physical_device);

        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&[queue_info])
            .enabled_extension_names(&extension_names)
            .enabled_features(&supported_features.features)
            .push_next(&mut desc_index_features)
            .build();

        let device = self
            .instance
            .create_device(self.physical_device, &create_info)?;

        let queue = unsafe { device.get_device_queue(self.queue_family, 0) };

        Ok(Device {
            instance: self.instance,
            device,
            queue,
        })
    }
}
