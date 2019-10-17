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

const FENCE_TIMEOUT: u64 = 100;

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
    pub fn queue(&self) -> vk::Queue {
        self.queue
    }

    pub fn queue_wait_idle(&self) -> Result<(), VulkanError> {
        unsafe { self.device.queue_wait_idle(self.queue) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))?;

        Ok(())
    }

    pub fn queue_submit(
        &self,
        submit_info: &[vk::SubmitInfo],
        fence: vk::Fence,
    ) -> Result<(), VulkanError> {
        unsafe { self.device.queue_submit(self.queue, submit_info, fence) }
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

    pub fn create_render_pass(
        &self,
        render_pass_info: &vk::RenderPassCreateInfo,
    ) -> Result<vk::RenderPass, VulkanError> {
        unsafe { self.device.create_render_pass(&render_pass_info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_render_pass(&self, render_pass: vk::RenderPass) {
        unsafe {
            self.device.destroy_render_pass(render_pass, None);
        }
    }

    pub fn create_image_view(
        &self,
        view_info: &vk::ImageViewCreateInfo,
    ) -> Result<vk::ImageView, VulkanError> {
        unsafe { self.device.create_image_view(view_info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_image_view(&self, image_view: vk::ImageView) {
        unsafe {
            self.device.destroy_image_view(image_view, None);
        }
    }

    pub fn create_image(&self, image_info: &vk::ImageCreateInfo) -> Result<vk::Image, VulkanError> {
        unsafe { self.device.create_image(&image_info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_image(&self, image: vk::Image) {
        unsafe {
            self.device.destroy_image(image, None);
        }
    }

    pub fn get_image_memory_requirements(&self, image: vk::Image) -> vk::MemoryRequirements {
        unsafe { self.device.get_image_memory_requirements(image) }
    }

    pub fn allocate_memory(
        &self,
        alloc_info: &vk::MemoryAllocateInfo,
    ) -> Result<vk::DeviceMemory, VulkanError> {
        unsafe { self.device.allocate_memory(&alloc_info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn free_memory(&self, memory: vk::DeviceMemory) {
        unsafe {
            self.device.free_memory(memory, None);
        }
    }

    pub fn bind_image_memory(
        &self,
        image: vk::Image,
        memory: vk::DeviceMemory,
    ) -> Result<(), VulkanError> {
        unsafe { self.device.bind_image_memory(image, memory, 0) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn create_frame_buffer(
        &self,
        info: &vk::FramebufferCreateInfo,
    ) -> Result<vk::Framebuffer, VulkanError> {
        unsafe { self.device.create_framebuffer(info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_frame_buffer(&self, frame_buffer: vk::Framebuffer) {
        unsafe { self.device.destroy_framebuffer(frame_buffer, None) }
    }

    pub fn create_descriptor_set_layout(
        &self,
        layout_info: &vk::DescriptorSetLayoutCreateInfo,
    ) -> Result<vk::DescriptorSetLayout, VulkanError> {
        unsafe { self.device.create_descriptor_set_layout(&layout_info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_descriptor_set_layout(&self, descriptor_set_layout: vk::DescriptorSetLayout) {
        unsafe {
            self.device
                .destroy_descriptor_set_layout(descriptor_set_layout, None);
        }
    }

    pub fn create_pipeline_layout(
        &self,
        info: &vk::PipelineLayoutCreateInfo,
    ) -> Result<vk::PipelineLayout, VulkanError> {
        unsafe { self.device.create_pipeline_layout(info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_pipeline_layout(&self, pipeline_layout: vk::PipelineLayout) {
        unsafe {
            self.device.destroy_pipeline_layout(pipeline_layout, None);
        }
    }

    pub fn create_graphics_pipelines(
        &self,
        pipeline_info: &[vk::GraphicsPipelineCreateInfo],
    ) -> Result<Vec<vk::Pipeline>, VulkanError> {
        unsafe {
            self.device
                .create_graphics_pipelines(vk::PipelineCache::null(), pipeline_info, None)
        }
        .map_err(|(_, err)| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_pipeline(&self, pipeline: vk::Pipeline) {
        unsafe {
            self.device.destroy_pipeline(pipeline, None);
        }
    }

    pub fn create_shader_module(
        &self,
        info: &vk::ShaderModuleCreateInfo,
    ) -> Result<vk::ShaderModule, VulkanError> {
        unsafe { self.device.create_shader_module(info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_shader_module(&self, shader_module: vk::ShaderModule) {
        unsafe { self.device.destroy_shader_module(shader_module, None) }
    }

    pub fn create_buffer(&self, info: &vk::BufferCreateInfo) -> Result<vk::Buffer, VulkanError> {
        unsafe { self.device.create_buffer(info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_buffer(&self, buffer: vk::Buffer) {
        unsafe {
            self.device.destroy_buffer(buffer, None);
        }
    }

    pub fn get_buffer_memory_requirements(&self, buffer: vk::Buffer) -> vk::MemoryRequirements {
        unsafe { self.device.get_buffer_memory_requirements(buffer) }
    }

    pub fn bind_buffer_memory(
        &self,
        buffer: vk::Buffer,
        memory: vk::DeviceMemory,
    ) -> Result<(), VulkanError> {
        unsafe { self.device.bind_buffer_memory(buffer, memory, 0) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn allocate_descriptor_sets(
        &self,
        info: &vk::DescriptorSetAllocateInfo,
    ) -> Result<Vec<vk::DescriptorSet>, VulkanError> {
        unsafe { self.device.allocate_descriptor_sets(info) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn free_descriptor_sets(
        &self,
        pool: vk::DescriptorPool,
        descriptor_sets: &[vk::DescriptorSet],
    ) {
        unsafe { self.device.free_descriptor_sets(pool, descriptor_sets) }
    }

    pub fn update_descriptor_sets(&self, descriptor_writes: &[vk::WriteDescriptorSet]) {
        unsafe { self.device.update_descriptor_sets(descriptor_writes, &[]) }
    }

    pub fn begin_command_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        begin_info: &vk::CommandBufferBeginInfo,
    ) -> Result<(), VulkanError> {
        unsafe { self.device.begin_command_buffer(command_buffer, begin_info) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn end_command_buffer(&self, command_buffer: vk::CommandBuffer) -> Result<(), VulkanError> {
        unsafe { self.device.end_command_buffer(command_buffer) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn cmd_pipeline_barrier(
        &self,
        command_buffer: vk::CommandBuffer,
        src_stage: vk::PipelineStageFlags,
        dst_stage: vk::PipelineStageFlags,
        dependency_flags: vk::DependencyFlags,
        memory_barriers: &[vk::MemoryBarrier],
        buffer_memory_barriers: &[vk::BufferMemoryBarrier],
        image_memory_barriers: &[vk::ImageMemoryBarrier],
    ) {
        unsafe {
            self.device.cmd_pipeline_barrier(
                command_buffer,
                src_stage,
                dst_stage,
                dependency_flags,
                memory_barriers,
                buffer_memory_barriers,
                image_memory_barriers,
            );
        }
    }

    pub fn wait_for_fences(&self, fences: &[vk::Fence]) -> Result<(), VulkanError> {
        unsafe { self.device.wait_for_fences(fences, true, FENCE_TIMEOUT) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn reset_fences(&self, fences: &[vk::Fence]) -> Result<(), VulkanError> {
        unsafe { self.device.reset_fences(fences) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn cmd_begin_render_pass(
        &self,
        command_buffer: vk::CommandBuffer,
        info: &vk::RenderPassBeginInfo,
    ) {
        unsafe {
            self.device
                .cmd_begin_render_pass(command_buffer, info, vk::SubpassContents::INLINE);
        }
    }

    pub fn cmd_next_subpass(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device
                .cmd_next_subpass(command_buffer, vk::SubpassContents::INLINE);
        }
    }

    pub fn cmd_end_render_pass(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device.cmd_end_render_pass(command_buffer);
        }
    }

    pub fn cmd_bind_pipeline(&self, command_buffer: vk::CommandBuffer, pipeline: vk::Pipeline) {
        unsafe {
            self.device
                .cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline)
        }
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

        let supported_features = vk::PhysicalDeviceFeatures::builder()
            .sampler_anisotropy(true)
            .build();

        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&[queue_info])
            .enabled_extension_names(&extension_names)
            .enabled_features(&supported_features)
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
