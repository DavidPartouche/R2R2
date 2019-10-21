use std::os::raw::c_void;
use std::ptr;

use ash::vk;

use crate::buffer::{Buffer, BufferBuilder, BufferType};
use crate::errors::VulkanError;
use crate::pipeline::Pipeline;
use crate::ray_tracing::RayTracing;
use crate::vulkan_context::VulkanContext;

pub struct ShaderBindingTable {
    _sbt_buffer: Buffer,
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
        let prog_id_size = self.ray_tracing.get_properties().shader_group_handle_size;

        let entry_size = (prog_id_size + (prog_id_size % 16)) as vk::DeviceSize;
        let ray_gen_entry_size = entry_size;
        let miss_entry_size = entry_size;
        let hit_group_entry_size = entry_size;
        let sbt_size = ray_gen_entry_size + miss_entry_size + hit_group_entry_size;

        let sbt_buffer = BufferBuilder::new(self.context)
            .with_type(BufferType::ShaderBindingTable)
            .with_size(sbt_size)
            .build()?;

        let group_count: u32 = 3;

        let mut shader_handle_storage = Vec::with_capacity((group_count * prog_id_size) as usize);
        self.ray_tracing.get_ray_tracing_shader_group_handles(
            self.pipeline.get(),
            0,
            group_count,
            &mut shader_handle_storage,
        )?;

        let data = self
            .context
            .device
            .map_memory(sbt_buffer.get_memory(), sbt_size)?;

        self.copy_shader_data(
            shader_handle_storage.as_ptr() as *const c_void,
            data,
            self.pipeline.ray_gen_index,
            prog_id_size,
        );
        let data = unsafe { data.offset(ray_gen_entry_size as isize) };

        self.copy_shader_data(
            shader_handle_storage.as_ptr() as *const c_void,
            data,
            self.pipeline.miss_index,
            prog_id_size,
        );
        let data = unsafe { data.offset(miss_entry_size as isize) };

        self.copy_shader_data(
            shader_handle_storage.as_ptr() as *const c_void,
            data,
            self.pipeline.hit_group_index,
            prog_id_size,
        );

        self.context.device.unmap_memory(sbt_buffer.get_memory());

        Ok(ShaderBindingTable {
            _sbt_buffer: sbt_buffer,
        })
    }

    fn copy_shader_data(
        &self,
        shader_handle_storage: *const c_void,
        data: *mut c_void,
        shader_index: u32,
        prog_id_size: u32,
    ) {
        let src = unsafe { shader_handle_storage.offset((shader_index * prog_id_size) as isize) };
        unsafe {
            ptr::copy(src, data, prog_id_size as usize);
        }
    }
}
