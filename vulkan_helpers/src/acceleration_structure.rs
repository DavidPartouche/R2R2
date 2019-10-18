use std::rc::Rc;

use ash::vk;

use crate::buffer::{Buffer, BufferBuilder, BufferType};
use crate::errors::VulkanError;
use crate::pipeline_context::GraphicsPipelineContext;
use crate::ray_tracing::RayTracing;
use crate::vulkan_context::VulkanContext;

pub struct AccelerationStructure {
    ray_tracing: Rc<RayTracing>,
    scratch_buffer: Buffer,
    result_buffer: Buffer,
    //    instances_buffer: Buffer,
    acc_structure: vk::AccelerationStructureNV,
}

impl Drop for AccelerationStructure {
    fn drop(&mut self) {
        self.ray_tracing
            .destroy_acceleration_structure(self.acc_structure);
    }
}

pub struct AccelerationStructureBuilder<'a> {
    context: &'a VulkanContext,
    pipeline_context: &'a GraphicsPipelineContext,
    command_buffer: Option<vk::CommandBuffer>,
    bottom_level_as: Vec<BottomLevelAccelerationStructure>,
    allow_update: bool,
}

impl<'a> AccelerationStructureBuilder<'a> {
    pub fn new(context: &'a VulkanContext, pipeline_context: &'a GraphicsPipelineContext) -> Self {
        AccelerationStructureBuilder {
            context,
            pipeline_context,
            command_buffer: None,
            bottom_level_as: vec![],
            allow_update: false,
        }
    }

    pub fn with_bottom_level_as(
        mut self,
        bottom_level_as: Vec<BottomLevelAccelerationStructure>,
    ) -> Self {
        self.bottom_level_as = bottom_level_as;
        self
    }

    pub fn with_allow_update(mut self, allow_update: bool) -> Self {
        self.allow_update = allow_update;
        self
    }

    pub fn with_command_buffer(mut self, command_buffer: vk::CommandBuffer) -> Self {
        self.command_buffer = Some(command_buffer);
        self
    }

    pub fn build(self) -> Result<AccelerationStructure, VulkanError> {
        let flags = if self.allow_update {
            vk::BuildAccelerationStructureFlagsNV::ALLOW_UPDATE
        } else {
            vk::BuildAccelerationStructureFlagsNV::empty()
        };

        let as_info = vk::AccelerationStructureInfoNV::builder()
            .ty(vk::AccelerationStructureTypeNV::BOTTOM_LEVEL)
            .flags(flags)
            .instance_count(0)
            .geometries(self.bottom_level_as.as_slice())
            .build();
        let as_create_info = vk::AccelerationStructureCreateInfoNV::builder()
            .info(as_info)
            .compacted_size(0)
            .build();

        let acc_structure = self
            .pipeline_context
            .ray_tracing
            .create_acceleration_structure(&as_create_info)?;

        let (scratch_size, result_size) = self.compute_as_buffer_sizes(acc_structure);

        let scratch_buffer = BufferBuilder::new(self.context)
            .with_type(BufferType::RayTracing)
            .with_size(scratch_size)
            .build()?;

        let result_buffer = BufferBuilder::new(self.context)
            .with_type(BufferType::RayTracing)
            .with_size(result_size)
            .build()?;

        self.generate(acc_structure, &scratch_buffer, &result_buffer, flags)?;

        Ok(AccelerationStructure {
            ray_tracing: Rc::clone(&self.pipeline_context.ray_tracing),
            acc_structure,
            scratch_buffer,
            result_buffer,
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
        self.pipeline_context
            .ray_tracing
            .get_acceleration_structure_memory_requirements(&mem_requirements_info)
    }

    fn generate(
        &self,
        acc_structure: vk::AccelerationStructureNV,
        scratch_buffer: &Buffer,
        result_buffer: &Buffer,
        flags: vk::BuildAccelerationStructureFlagsNV,
    ) -> Result<(), VulkanError> {
        let bind_info = vk::BindAccelerationStructureMemoryInfoNV::builder()
            .acceleration_structure(acc_structure)
            .memory(result_buffer.get_memory())
            .memory_offset(0)
            .build();

        self.pipeline_context
            .ray_tracing
            .bind_acceleration_structure_memory(&[bind_info])?;

        let build_info = vk::AccelerationStructureInfoNV::builder()
            .flags(flags)
            .ty(vk::AccelerationStructureTypeNV::BOTTOM_LEVEL)
            .geometries(self.bottom_level_as.as_slice())
            .instance_count(0)
            .build();

        self.pipeline_context
            .ray_tracing
            .cmd_build_acceleration_structure(
                self.command_buffer.unwrap(),
                &build_info,
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

        self.pipeline_context.device.cmd_pipeline_barrier(
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

pub type BottomLevelAccelerationStructure = vk::GeometryNV;

pub struct BottomLevelAccelerationStructureBuilder {
    vertex_buffer: Option<vk::Buffer>,
    vertex_offset: vk::DeviceSize,
    vertex_count: u32,
    vertex_size: vk::DeviceSize,
    index_buffer: Option<vk::Buffer>,
    index_offset: vk::DeviceSize,
    index_count: u32,
    transform_buffer: Option<vk::Buffer>,
    transform_offset: vk::DeviceSize,
    opaque: bool,
}

impl Default for BottomLevelAccelerationStructureBuilder {
    fn default() -> Self {
        BottomLevelAccelerationStructureBuilder {
            vertex_buffer: None,
            vertex_offset: 0,
            vertex_count: 0,
            vertex_size: 0,
            index_buffer: None,
            index_offset: 0,
            index_count: 0,
            transform_buffer: None,
            transform_offset: 0,
            opaque: false,
        }
    }
}

impl BottomLevelAccelerationStructureBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_vertex_buffer(mut self, buffer: vk::Buffer) -> Self {
        self.vertex_buffer = Some(buffer);
        self
    }

    pub fn with_vertex_offset(mut self, offset: u32) -> Self {
        self.vertex_offset = offset as vk::DeviceSize;
        self
    }

    pub fn with_vertex_count(mut self, count: u32) -> Self {
        self.vertex_count = count;
        self
    }

    pub fn with_vertex_size(mut self, size: u32) -> Self {
        self.vertex_size = size as vk::DeviceSize;
        self
    }

    pub fn with_index_buffer(mut self, buffer: vk::Buffer) -> Self {
        self.index_buffer = Some(buffer);
        self
    }

    pub fn with_index_offset(mut self, offset: u32) -> Self {
        self.index_offset = offset as vk::DeviceSize;
        self
    }

    pub fn with_index_count(mut self, count: u32) -> Self {
        self.index_count = count;
        self
    }

    pub fn with_transform_buffer(mut self, buffer: vk::Buffer) -> Self {
        self.transform_buffer = Some(buffer);
        self
    }

    pub fn with_transform_offset(mut self, offset: u32) -> Self {
        self.transform_offset = offset as vk::DeviceSize;
        self
    }

    pub fn build(self) -> BottomLevelAccelerationStructure {
        let triangles = vk::GeometryTrianglesNV::builder()
            .vertex_data(self.vertex_buffer.unwrap())
            .vertex_offset(self.vertex_offset)
            .vertex_count(self.vertex_count)
            .vertex_stride(self.vertex_size)
            .vertex_format(vk::Format::R32G32B32_SFLOAT)
            .index_data(self.index_buffer.unwrap())
            .index_offset(self.index_offset)
            .index_count(self.index_count)
            .index_type(vk::IndexType::UINT32)
            .transform_data(self.transform_buffer.unwrap_or_else(vk::Buffer::null))
            .transform_offset(self.transform_offset)
            .build();

        let flags = if self.opaque {
            vk::GeometryFlagsNV::OPAQUE
        } else {
            vk::GeometryFlagsNV::empty()
        };

        vk::GeometryNV::builder()
            .geometry_type(vk::GeometryTypeNV::TRIANGLES)
            .geometry(
                vk::GeometryDataNV::builder()
                    .triangles(triangles)
                    .aabbs(vk::GeometryAABBNV::default())
                    .build(),
            )
            .flags(flags)
            .build()
    }
}
