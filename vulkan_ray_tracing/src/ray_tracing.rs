use ash::vk;
use vulkan_bootstrap::errors::VulkanError;
use vulkan_bootstrap::vulkan_context::VulkanContext;

pub struct RayTracing {
    ray_tracing: ash::extensions::nv::RayTracing,
    ray_tracing_properties: vk::PhysicalDeviceRayTracingPropertiesNV,
}

impl RayTracing {
    pub fn get_properties(&self) -> vk::PhysicalDeviceRayTracingPropertiesNV {
        self.ray_tracing_properties
    }

    pub fn create_acceleration_structure(
        &self,
        info: &vk::AccelerationStructureCreateInfoNV,
    ) -> Result<vk::AccelerationStructureNV, VulkanError> {
        unsafe { self.ray_tracing.create_acceleration_structure(info, None) }
            .map_err(|err| VulkanError::PipelineError(err.to_string()))
    }

    pub fn destroy_acceleration_structure(
        &self,
        acceleration_structure: vk::AccelerationStructureNV,
    ) {
        unsafe {
            self.ray_tracing
                .destroy_acceleration_structure(acceleration_structure, None);
        }
    }

    pub fn get_acceleration_structure_handle(
        &self,
        accel_struct: vk::AccelerationStructureNV,
    ) -> Result<u64, VulkanError> {
        unsafe {
            self.ray_tracing
                .get_acceleration_structure_handle(accel_struct)
        }
        .map_err(|err| VulkanError::PipelineError(err.to_string()))
    }

    pub fn get_acceleration_structure_memory_requirements(
        &self,
        info: &vk::AccelerationStructureMemoryRequirementsInfoNV,
    ) -> vk::MemoryRequirements2 {
        unsafe {
            self.ray_tracing
                .get_acceleration_structure_memory_requirements(info)
        }
    }

    pub fn bind_acceleration_structure_memory(
        &self,
        info: &[vk::BindAccelerationStructureMemoryInfoNV],
    ) -> Result<(), VulkanError> {
        unsafe { self.ray_tracing.bind_acceleration_structure_memory(info) }
            .map_err(|err| VulkanError::PipelineError(err.to_string()))
    }

    pub fn cmd_build_acceleration_structure(
        &self,
        command_buffer: vk::CommandBuffer,
        info: &vk::AccelerationStructureInfoNV,
        instance_buffer: vk::Buffer,
        acceleration_structure: vk::AccelerationStructureNV,
        scratch_buffer: vk::Buffer,
        scratch_offset: vk::DeviceSize,
    ) {
        unsafe {
            self.ray_tracing.cmd_build_acceleration_structure(
                command_buffer,
                info,
                instance_buffer,
                0,
                false,
                acceleration_structure,
                vk::AccelerationStructureNV::null(),
                scratch_buffer,
                scratch_offset,
            )
        }
    }

    pub fn create_ray_tracing_pipelines(
        &self,
        info: &[vk::RayTracingPipelineCreateInfoNV],
    ) -> Result<Vec<vk::Pipeline>, VulkanError> {
        unsafe {
            self.ray_tracing
                .create_ray_tracing_pipelines(vk::PipelineCache::null(), info, None)
        }
        .map_err(|err| VulkanError::PipelineError(err.to_string()))
    }

    pub fn get_ray_tracing_shader_group_handles(
        &self,
        pipeline: vk::Pipeline,
        first_group: u32,
        group_count: u32,
        data: &mut [u8],
    ) -> Result<(), VulkanError> {
        unsafe {
            self.ray_tracing.get_ray_tracing_shader_group_handles(
                pipeline,
                first_group,
                group_count,
                data,
            )
        }
        .map_err(|err| VulkanError::PipelineError(err.to_string()))
    }

    pub fn cmd_trace_rays(
        &self,
        command_buffer: vk::CommandBuffer,
        ray_gen_sbt: vk::Buffer,
        ray_gen_offset: vk::DeviceSize,
        miss_sbt: vk::Buffer,
        miss_offset: vk::DeviceSize,
        miss_stride: vk::DeviceSize,
        hit_group_sbt: vk::Buffer,
        hit_group_offset: vk::DeviceSize,
        hit_group_stride: vk::DeviceSize,
        width: u32,
        height: u32,
        depth: u32,
    ) {
        unsafe {
            self.ray_tracing.cmd_trace_rays(
                command_buffer,
                ray_gen_sbt,
                ray_gen_offset,
                miss_sbt,
                miss_offset,
                miss_stride,
                hit_group_sbt,
                hit_group_offset,
                hit_group_stride,
                vk::Buffer::null(),
                0,
                0,
                width,
                height,
                depth,
            )
        }
    }
}

pub struct RayTracingBuilder<'a> {
    context: &'a VulkanContext,
}

impl<'a> RayTracingBuilder<'a> {
    pub fn new(context: &'a VulkanContext) -> Self {
        RayTracingBuilder { context }
    }

    pub fn build(self) -> Result<RayTracing, VulkanError> {
        let mut ray_tracing_properties = vk::PhysicalDeviceRayTracingPropertiesNV::builder()
            .max_recursion_depth(0)
            .shader_group_handle_size(0)
            .build();

        let mut props = vk::PhysicalDeviceProperties2::builder()
            .push_next(&mut ray_tracing_properties)
            .build();

        self.context
            .get_instance()
            .get_physical_device_properties2(self.context.get_physical_device().get(), &mut props);

        let ray_tracing = ash::extensions::nv::RayTracing::new(
            self.context.get_instance().get(),
            self.context.get_device().get(),
        );

        Ok(RayTracing {
            ray_tracing,
            ray_tracing_properties,
        })
    }
}
