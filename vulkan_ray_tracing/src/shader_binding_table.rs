use std::os::raw::c_void;

use ash::vk;
use vulkan_bootstrap::buffer::{Buffer, BufferBuilder, BufferType};
use vulkan_bootstrap::errors::VulkanError;
use vulkan_bootstrap::vulkan_context::VulkanContext;

use crate::pipeline::Pipeline;
use crate::ray_tracing::RayTracing;

pub struct ShaderBindingTable {
    sbt_buffer: Buffer,
    pub ray_gen_entry_size: vk::DeviceSize,
    pub ray_gen_offset: vk::DeviceSize,
    pub miss_entry_size: vk::DeviceSize,
    pub miss_offset: vk::DeviceSize,
    pub hit_group_entry_size: vk::DeviceSize,
    pub hit_group_offset: vk::DeviceSize,
}

impl ShaderBindingTable {
    pub fn get(&self) -> vk::Buffer {
        self.sbt_buffer.get()
    }
}

pub struct ShaderBindingTableBuilder<'a> {
    context: &'a VulkanContext,
    ray_tracing: &'a RayTracing,
    pipeline: &'a Pipeline,
}

impl<'a> ShaderBindingTableBuilder<'a> {
    pub fn new(
        context: &'a VulkanContext,
        ray_tracing: &'a RayTracing,
        pipeline: &'a Pipeline,
    ) -> Self {
        ShaderBindingTableBuilder {
            context,
            ray_tracing,
            pipeline,
        }
    }

    pub fn build(self) -> Result<ShaderBindingTable, VulkanError> {
        let ray_gen = vec![self.pipeline.ray_gen_index];
        let miss = vec![self.pipeline.miss_index, self.pipeline.shadow_miss_index];
        let hit_group = vec![
            self.pipeline.hit_group_index,
            self.pipeline.shadow_hit_group_index,
        ];

        let prog_id_size = self.ray_tracing.get_properties().shader_group_handle_size;
        let entry_size = (prog_id_size + (prog_id_size % 16)) as vk::DeviceSize;

        let ray_gen_entry_size = entry_size;
        let miss_entry_size = entry_size;
        let hit_group_entry_size = entry_size;

        let sbt_size = ray_gen_entry_size * ray_gen.len() as u64
            + miss_entry_size * miss.len() as u64
            + hit_group_entry_size * hit_group.len() as u64;

        let sbt_buffer = BufferBuilder::new(self.context)
            .with_type(BufferType::ShaderBindingTable)
            .with_size(sbt_size)
            .build()?;

        let group_count = (ray_gen.len() + miss.len() + hit_group.len()) as u32;
        let mut shader_handle_storage = vec![0u8; (group_count * prog_id_size) as usize];

        self.ray_tracing.get_ray_tracing_shader_group_handles(
            self.pipeline.get(),
            0,
            group_count,
            &mut shader_handle_storage,
        )?;

        let data = self
            .context
            .get_device()
            .map_memory(sbt_buffer.get_memory(), sbt_size)?;

        unsafe {
            std::ptr::copy(
                shader_handle_storage.as_ptr() as *const c_void,
                data,
                sbt_size as usize,
            );
        }

        self.context
            .get_device()
            .unmap_memory(sbt_buffer.get_memory());

        let ray_gen_offset = 0;
        let miss_offset = ray_gen_entry_size * ray_gen.len() as vk::DeviceSize;
        let hit_group_offset = miss_offset + miss_entry_size * miss.len() as vk::DeviceSize;

        Ok(ShaderBindingTable {
            sbt_buffer,
            ray_gen_entry_size,
            ray_gen_offset,
            miss_entry_size,
            miss_offset,
            hit_group_entry_size,
            hit_group_offset,
        })
    }
}
