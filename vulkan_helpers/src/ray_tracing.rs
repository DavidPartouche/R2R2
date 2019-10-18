use ash::vk;

use crate::errors::VulkanError;
use crate::vulkan_context::VulkanContext;

pub struct RayTracing;

pub struct RayTracingBuilder<'a> {
    context: &'a VulkanContext,
}

impl<'a> RayTracingBuilder<'a> {
    pub fn new(context: &'a VulkanContext) -> Self {
        RayTracingBuilder { context }
    }

    pub fn build(self) -> Result<RayTracing, VulkanError> {
        let mut raytracing_properties = vk::PhysicalDeviceRayTracingPropertiesNV::builder()
            .max_recursion_depth(0)
            .shader_group_handle_size(0)
            .build();

        let mut props = vk::PhysicalDeviceProperties2::builder()
            .push_next(&mut raytracing_properties)
            .build();

        let props = self
            .context
            .instance
            .get_physical_device_properties2(self.context.physical_device, &mut props);

        Ok(RayTracing)
    }
}
