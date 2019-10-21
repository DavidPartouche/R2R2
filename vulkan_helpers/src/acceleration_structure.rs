use std::os::raw::c_void;
use std::rc::Rc;
use std::{mem, ptr};

use ash::vk;

use crate::bottom_level_acceleration_structure::BottomLevelAccelerationStructure;
use crate::buffer::{Buffer, BufferBuilder, BufferType};
use crate::errors::VulkanError;
use crate::glm;
use crate::ray_tracing::RayTracing;
use crate::vulkan_context::VulkanContext;

pub struct Instance {
    pub bottom_level_as: vk::AccelerationStructureNV,
    pub transform: glm::Mat4,
    pub instance_id: u32,
    pub hit_group_index: u32,
}

// TODO: Change some values to u24
#[repr(C, packed)]
struct VulkanGeometryInstance {
    transform: [f32; 12],
    instance_id: u32,
    mask: u8,
    instance_offset: u32,
    flags: u32,
    acceleration_structure_handle: u64,
}

pub struct AccelerationStructure {
    ray_tracing: Rc<RayTracing>,
    _scratch_buffer: Buffer,
    _result_buffer: Buffer,
    _instances_buffer: Option<Buffer>,
    acc_structure: vk::AccelerationStructureNV,
}

impl Drop for AccelerationStructure {
    fn drop(&mut self) {
        self.ray_tracing
            .destroy_acceleration_structure(self.acc_structure);
    }
}

impl AccelerationStructure {
    pub fn get(&self) -> vk::AccelerationStructureNV {
        self.acc_structure
    }
}

pub struct AccelerationStructureBuilder<'a> {
    context: &'a VulkanContext,
    ray_tracing: Rc<RayTracing>,
    command_buffer: Option<vk::CommandBuffer>,
    bottom_level_as: Option<&'a [BottomLevelAccelerationStructure]>,
    top_level_as: Option<&'a [Instance]>,
}

impl<'a> AccelerationStructureBuilder<'a> {
    pub fn new(context: &'a VulkanContext, ray_tracing: Rc<RayTracing>) -> Self {
        AccelerationStructureBuilder {
            context,
            ray_tracing,
            command_buffer: None,
            bottom_level_as: None,
            top_level_as: None,
        }
    }

    pub fn with_bottom_level_as(
        mut self,
        bottom_level_as: &'a [BottomLevelAccelerationStructure],
    ) -> Self {
        self.bottom_level_as = Some(bottom_level_as);
        self
    }

    pub fn with_top_level_as(mut self, instances: &'a [Instance]) -> Self {
        self.top_level_as = Some(instances);
        self
    }

    pub fn with_command_buffer(mut self, command_buffer: vk::CommandBuffer) -> Self {
        self.command_buffer = Some(command_buffer);
        self
    }

    pub fn build(self) -> Result<AccelerationStructure, VulkanError> {
        let as_info = if self.bottom_level_as.is_some() {
            vk::AccelerationStructureInfoNV::builder()
                .ty(vk::AccelerationStructureTypeNV::BOTTOM_LEVEL)
                .flags(vk::BuildAccelerationStructureFlagsNV::empty())
                .instance_count(0)
                .geometries(self.bottom_level_as.unwrap())
                .build()
        } else {
            vk::AccelerationStructureInfoNV::builder()
                .ty(vk::AccelerationStructureTypeNV::TOP_LEVEL)
                .flags(vk::BuildAccelerationStructureFlagsNV::empty())
                .instance_count(self.top_level_as.unwrap().len() as u32)
                .geometries(&[])
                .build()
        };

        let as_create_info = vk::AccelerationStructureCreateInfoNV::builder()
            .info(as_info)
            .compacted_size(0)
            .build();

        let acc_structure = self
            .ray_tracing
            .create_acceleration_structure(&as_create_info)?;

        let (scratch_size, result_size) = self.compute_as_buffer_sizes(acc_structure);

        let instances_size = if self.top_level_as.is_some() {
            (self.top_level_as.unwrap().len() * mem::size_of::<VulkanGeometryInstance>())
                as vk::DeviceSize
        } else {
            0
        };

        let scratch_buffer = BufferBuilder::new(self.context)
            .with_type(BufferType::RayTracing)
            .with_size(scratch_size)
            .build()?;

        let result_buffer = BufferBuilder::new(self.context)
            .with_type(BufferType::RayTracing)
            .with_size(result_size)
            .build()?;

        let instances_buffer = if self.bottom_level_as.is_some() {
            None
        } else {
            Some(
                BufferBuilder::new(self.context)
                    .with_type(BufferType::RayTracingInstance)
                    .with_size(instances_size)
                    .build()?,
            )
        };

        self.generate(
            acc_structure,
            &scratch_buffer,
            &result_buffer,
            instances_buffer.as_ref(),
        )?;

        Ok(AccelerationStructure {
            ray_tracing: self.ray_tracing,
            acc_structure,
            _scratch_buffer: scratch_buffer,
            _result_buffer: result_buffer,
            _instances_buffer: instances_buffer,
        })
    }

    fn compute_as_buffer_sizes(
        &self,
        acc_structure: vk::AccelerationStructureNV,
    ) -> (vk::DeviceSize, vk::DeviceSize) {
        let mem_requirements = self.get_memory_requirements(
            acc_structure,
            vk::AccelerationStructureMemoryRequirementsTypeNV::OBJECT,
        );
        let result_size = mem_requirements.memory_requirements.size;

        let mem_requirements = self.get_memory_requirements(
            acc_structure,
            vk::AccelerationStructureMemoryRequirementsTypeNV::BUILD_SCRATCH,
        );
        let scratch_size = mem_requirements.memory_requirements.size;

        let mem_requirements = self.get_memory_requirements(
            acc_structure,
            vk::AccelerationStructureMemoryRequirementsTypeNV::UPDATE_SCRATCH,
        );
        let scratch_size = scratch_size.max(mem_requirements.memory_requirements.size);

        (scratch_size, result_size)
    }

    fn get_memory_requirements(
        &self,
        acc_structure: vk::AccelerationStructureNV,
        ty: vk::AccelerationStructureMemoryRequirementsTypeNV,
    ) -> vk::MemoryRequirements2 {
        let mem_requirements_info = vk::AccelerationStructureMemoryRequirementsInfoNV::builder()
            .acceleration_structure(acc_structure)
            .ty(ty)
            .build();
        self.ray_tracing
            .get_acceleration_structure_memory_requirements(&mem_requirements_info)
    }

    fn generate(
        &self,
        acc_structure: vk::AccelerationStructureNV,
        scratch_buffer: &Buffer,
        result_buffer: &Buffer,
        instances_buffer: Option<&Buffer>,
    ) -> Result<(), VulkanError> {
        if let Some(top_level_as) = self.top_level_as {
            let mut geometry_instances = Vec::with_capacity(top_level_as.len());
            for tlas in top_level_as.iter() {
                let handle = self
                    .ray_tracing
                    .get_acceleration_structure_handle(tlas.bottom_level_as)?;

                let mut g_inst = VulkanGeometryInstance {
                    transform: [0.0; 12],
                    instance_id: tlas.instance_id,
                    mask: std::u8::MAX,
                    instance_offset: tlas.hit_group_index,
                    flags: vk::GeometryInstanceFlagsNV::TRIANGLE_CULL_DISABLE.as_raw(),
                    acceleration_structure_handle: handle,
                };

                let src = glm::transpose(&tlas.transform).as_ptr() as *const f32;
                unsafe {
                    let dst = g_inst.transform.as_mut_ptr();
                    ptr::copy(src, dst, mem::size_of::<[f32; 12]>());
                }

                geometry_instances.push(g_inst);
            }

            instances_buffer
                .unwrap()
                .copy_data(geometry_instances.as_ptr() as *const c_void)?;
        }

        let bind_info = vk::BindAccelerationStructureMemoryInfoNV::builder()
            .acceleration_structure(acc_structure)
            .memory(result_buffer.get_memory())
            .memory_offset(0)
            .build();

        self.ray_tracing
            .bind_acceleration_structure_memory(&[bind_info])?;

        let build_info = if self.bottom_level_as.is_some() {
            vk::AccelerationStructureInfoNV::builder()
                .flags(vk::BuildAccelerationStructureFlagsNV::empty())
                .ty(vk::AccelerationStructureTypeNV::BOTTOM_LEVEL)
                .geometries(self.bottom_level_as.unwrap())
                .instance_count(0)
                .build()
        } else {
            vk::AccelerationStructureInfoNV::builder()
                .flags(vk::BuildAccelerationStructureFlagsNV::empty())
                .ty(vk::AccelerationStructureTypeNV::TOP_LEVEL)
                .instance_count(self.top_level_as.unwrap().len() as u32)
                .build()
        };

        let instance_buffer = match instances_buffer {
            Some(buffer) => buffer.get(),
            None => vk::Buffer::null(),
        };

        self.ray_tracing.cmd_build_acceleration_structure(
            self.command_buffer.unwrap(),
            &build_info,
            instance_buffer,
            acc_structure,
            scratch_buffer.get(),
            0,
        );

        let memory_barrier = vk::MemoryBarrier::builder()
            .src_access_mask(
                vk::AccessFlags::ACCELERATION_STRUCTURE_WRITE_NV
                    | vk::AccessFlags::ACCELERATION_STRUCTURE_READ_NV,
            )
            .dst_access_mask(
                vk::AccessFlags::ACCELERATION_STRUCTURE_WRITE_NV
                    | vk::AccessFlags::ACCELERATION_STRUCTURE_READ_NV,
            )
            .build();

        self.context.device.cmd_pipeline_barrier(
            self.command_buffer.unwrap(),
            vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_NV,
            vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_NV,
            vk::DependencyFlags::empty(),
            &[memory_barrier],
            &[],
            &[],
        );

        Ok(())
    }
}
