use core::ptr;
use std::os::raw::c_void;
use std::rc::Rc;

use ash::vk;

use crate::device::VulkanDevice;
use crate::errors::VulkanError;
use crate::vulkan_context::VulkanContext;

pub enum BufferType {
    Index,
    RayTracing,
    RayTracingInstance,
    ShaderBindingTable,
    Staging,
    Storage,
    Uniform,
    Vertex,
}

pub struct Buffer {
    device: Rc<VulkanDevice>,
    buffer: vk::Buffer,
    buffer_memory: vk::DeviceMemory,
    buffer_size: vk::DeviceSize,
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

    pub fn get_memory(&self) -> vk::DeviceMemory {
        self.buffer_memory
    }

    pub fn copy_data(&self, buffer: *const c_void) -> Result<(), VulkanError> {
        let data = self
            .device
            .map_memory(self.buffer_memory, self.buffer_size)?;
        unsafe {
            ptr::copy(buffer, data, self.buffer_size as usize);
        }
        self.device.unmap_memory(self.buffer_memory);

        Ok(())
    }
}

pub struct BufferBuilder<'a> {
    context: &'a VulkanContext,
    ty: BufferType,
    buffer_size: vk::DeviceSize,
}

impl<'a> BufferBuilder<'a> {
    pub fn new(context: &'a VulkanContext) -> Self {
        BufferBuilder {
            context,
            ty: BufferType::Uniform,
            buffer_size: 0,
        }
    }

    pub fn with_type(mut self, ty: BufferType) -> Self {
        self.ty = ty;
        self
    }

    pub fn with_size(mut self, size: vk::DeviceSize) -> Self {
        self.buffer_size = size;
        self
    }

    pub fn build(self) -> Result<Buffer, VulkanError> {
        let usage = match &self.ty {
            BufferType::Index => {
                vk::BufferUsageFlags::INDEX_BUFFER
                    | vk::BufferUsageFlags::TRANSFER_DST
                    | vk::BufferUsageFlags::STORAGE_BUFFER
            }
            BufferType::RayTracing => vk::BufferUsageFlags::RAY_TRACING_NV,
            BufferType::RayTracingInstance => vk::BufferUsageFlags::RAY_TRACING_NV,
            BufferType::ShaderBindingTable => vk::BufferUsageFlags::TRANSFER_SRC,
            BufferType::Staging => vk::BufferUsageFlags::TRANSFER_SRC,
            BufferType::Storage => {
                vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST
            }
            BufferType::Uniform => vk::BufferUsageFlags::UNIFORM_BUFFER,
            BufferType::Vertex => {
                vk::BufferUsageFlags::VERTEX_BUFFER
                    | vk::BufferUsageFlags::TRANSFER_DST
                    | vk::BufferUsageFlags::STORAGE_BUFFER
            }
        };

        let properties = match &self.ty {
            BufferType::Index => vk::MemoryPropertyFlags::DEVICE_LOCAL,
            BufferType::RayTracing => vk::MemoryPropertyFlags::DEVICE_LOCAL,
            BufferType::RayTracingInstance => {
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
            }
            BufferType::ShaderBindingTable => vk::MemoryPropertyFlags::HOST_VISIBLE,
            BufferType::Staging => {
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
            }
            BufferType::Storage => {
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
            }
            BufferType::Uniform => {
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
            }
            BufferType::Vertex => vk::MemoryPropertyFlags::DEVICE_LOCAL,
        };

        let buffer_info = vk::BufferCreateInfo::builder()
            .size(self.buffer_size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();

        let buffer = self.context.device.create_buffer(&buffer_info)?;

        let mem_requirements = self.context.device.get_buffer_memory_requirements(buffer);

        let memory_type_index = self
            .find_memory_type(mem_requirements.memory_type_bits, properties)
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
            buffer_size: self.buffer_size,
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
            .get_physical_device_memory_properties(self.context.physical_device.get());

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
