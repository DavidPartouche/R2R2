use std::rc::Rc;

use ash::vk;

use crate::device::VulkanDevice;
use crate::errors::VulkanError;
use crate::queue_family::QueueFamily;

pub struct CommandBuffers {
    device: Rc<VulkanDevice>,
    command_pools: Vec<vk::CommandPool>,
    command_buffers: Vec<vk::CommandBuffer>,
    fences: Vec<vk::Fence>,
    present_complete_semaphores: Vec<vk::Semaphore>,
    render_complete_semaphores: Vec<vk::Semaphore>,
}

impl Drop for CommandBuffers {
    fn drop(&mut self) {
        for render_complete_semaphore in self.render_complete_semaphores.iter() {
            self.device.destroy_semaphore(*render_complete_semaphore);
        }
        for present_complete_semaphore in self.present_complete_semaphores.iter() {
            self.device.destroy_semaphore(*present_complete_semaphore);
        }
        for fence in self.fences.iter() {
            self.device.destroy_fence(*fence);
        }
        for (command_pool, command_buffer) in
            self.command_pools.iter().zip(self.command_buffers.iter())
        {
            self.device
                .free_command_buffers(*command_pool, &[*command_buffer]);
            self.device.destroy_command_pool(*command_pool);
        }
    }
}

impl CommandBuffers {
    pub fn get(&self, index: usize) -> vk::CommandBuffer {
        self.command_buffers[index]
    }

    pub fn get_present_complete_semaphore(&self, index: usize) -> vk::Semaphore {
        self.present_complete_semaphores[index]
    }

    pub fn get_render_complete_semaphore(&self, index: usize) -> vk::Semaphore {
        self.render_complete_semaphores[index]
    }

    pub fn begin_single_time_commands(
        &self,
        frame_index: usize,
    ) -> Result<vk::CommandBuffer, VulkanError> {
        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(self.command_pools[frame_index])
            .command_buffer_count(1)
            .build();
        let command_buffer = self.device.allocate_command_buffers(&alloc_info)?[0];

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();
        self.device
            .begin_command_buffer(command_buffer, &begin_info)?;

        Ok(command_buffer)
    }

    pub fn end_single_time_commands(
        &self,
        command_buffer: vk::CommandBuffer,
        frame_index: usize,
    ) -> Result<(), VulkanError> {
        self.device.end_command_buffer(command_buffer)?;

        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(&[command_buffer])
            .build();

        self.device
            .queue_submit(&[submit_info], vk::Fence::null())?;
        self.device.queue_wait_idle()?;

        self.device
            .free_command_buffers(self.command_pools[frame_index], &[command_buffer]);

        Ok(())
    }

    pub fn wait_for_fence(&self, frame_index: usize) -> Result<(), VulkanError> {
        self.device.wait_for_fences(&[self.fences[frame_index]])
    }

    pub fn reset_fence(&self, frame_index: usize) -> Result<(), VulkanError> {
        self.device.reset_fences(&[self.fences[frame_index]])
    }

    pub fn begin_command_buffer(&self, frame_index: usize) -> Result<(), VulkanError> {
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();
        self.device
            .begin_command_buffer(self.command_buffers[frame_index], &begin_info)
    }

    pub fn end_command_buffer(&self, frame_index: usize) -> Result<(), VulkanError> {
        self.device
            .end_command_buffer(self.command_buffers[frame_index])
    }

    pub fn queue_submit(&self, frame_index: usize) -> Result<(), VulkanError> {
        let info = vk::SubmitInfo::builder()
            .wait_semaphores(&[self.present_complete_semaphores[frame_index]])
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(&[self.command_buffers[frame_index]])
            .signal_semaphores(&[self.render_complete_semaphores[frame_index]])
            .build();

        self.device.queue_submit(&[info], self.fences[frame_index])
    }

    pub fn copy_buffer(
        &self,
        src_buffer: vk::Buffer,
        dst_buffer: vk::Buffer,
        size: vk::DeviceSize,
    ) -> Result<(), VulkanError> {
        let command_buffer = self.begin_single_time_commands(0)?;
        let copy_region = vk::BufferCopy::builder().size(size).build();
        self.device
            .cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &[copy_region]);
        self.end_single_time_commands(command_buffer, 0)
    }
}

pub struct CommandBuffersBuilder {
    queue_family: QueueFamily,
    device: Rc<VulkanDevice>,
    buffer_count: usize,
}

impl CommandBuffersBuilder {
    pub fn new(queue_family: QueueFamily, device: Rc<VulkanDevice>) -> Self {
        Self {
            queue_family,
            device,
            buffer_count: 1,
        }
    }

    pub fn with_buffer_count(mut self, buffer_count: usize) -> Self {
        self.buffer_count = buffer_count;
        self
    }

    pub fn build(self) -> Result<CommandBuffers, VulkanError> {
        let mut command_pools = vec![];
        let mut command_buffers = vec![];
        let mut fences = vec![];
        let mut present_complete_semaphores = vec![];
        let mut render_complete_semaphores = vec![];

        for i in 0..self.buffer_count {
            let pool_info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(self.queue_family)
                .build();
            command_pools.push(self.device.create_command_pool(&pool_info)?);

            let alloc_info = vk::CommandBufferAllocateInfo::builder()
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_pool(command_pools[i])
                .command_buffer_count(1)
                .build();
            command_buffers.push(self.device.allocate_command_buffers(&alloc_info)?[0]);

            let fence_info = vk::FenceCreateInfo::builder()
                .flags(vk::FenceCreateFlags::SIGNALED)
                .build();
            fences.push(self.device.create_fence(&fence_info)?);

            let semaphore_info = vk::SemaphoreCreateInfo::builder().build();
            present_complete_semaphores.push(self.device.create_semaphore(&semaphore_info)?);
            render_complete_semaphores.push(self.device.create_semaphore(&semaphore_info)?);
        }
        Ok(CommandBuffers {
            device: self.device,
            command_pools,
            command_buffers,
            fences,
            present_complete_semaphores,
            render_complete_semaphores,
        })
    }
}
