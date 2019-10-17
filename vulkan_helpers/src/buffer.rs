use std::rc::Rc;

use ash::vk;

use crate::device::Device;
use crate::errors::VulkanError;
use crate::vulkan_context::VulkanContext;

pub struct Buffer {
    device: Rc<Device>,
    buffer: vk::Buffer,
    buffer_memory: vk::DeviceMemory,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        self.device.destroy_buffer(self.buffer);
        self.device.free_memory(self.buffer_memory);
    }
}

impl Buffer {
    pub fn get(&self) -> vk::Buffer {
        self.buffer
    }
}

pub struct BufferBuilder<'a> {
    context: &'a VulkanContext,
    size: vk::DeviceSize,
}

impl<'a> BufferBuilder<'a> {
    pub fn new(context: &'a VulkanContext) -> Self {
        BufferBuilder { context, size: 0 }
    }

    pub fn with_size(mut self, size: vk::DeviceSize) -> Self {
        self.size = size;
        self
    }

    pub fn build(self) -> Result<Buffer, VulkanError> {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(self.size)
            .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();

        let buffer = self.context.device.create_buffer(&buffer_info)?;

        let mem_requirements = self.context.device.get_buffer_memory_requirements(buffer);

        let memory_type_index = self
            .find_memory_type(
                mem_requirements.memory_type_bits,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )
            .ok_or_else(|| {
                VulkanError::VertexBufferCreationError(String::from("Cannot find a memory type"))
            })?;

        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(memory_type_index)
            .build();

        let buffer_memory = self.context.device.allocate_memory(&alloc_info)?;
        self.context
            .device
            .bind_buffer_memory(buffer, buffer_memory)?;

        Ok(Buffer {
            device: Rc::clone(&self.context.device),
            buffer,
            buffer_memory,
        })
    }

    fn find_memory_type(
        &self,
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> Option<u32> {
        let mem_properties = self
            .context
            .instance
            .get_physical_device_memory_properties(self.context.physical_device);

        for i in 0..mem_properties.memory_type_count {
            if type_filter & (1 << i) != 0
                && mem_properties.memory_types[i as usize]
                    .property_flags
                    .contains(properties)
            {
                return Some(i);
            }
        }

        None
    }
}
