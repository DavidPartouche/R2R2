use std::rc::Rc;

use ash::vk;

use crate::command_buffers::CommandBuffers;
use crate::device::Device;
use crate::errors::VulkanError;
use crate::instance::Instance;
use crate::physical_device::PhysicalDevice;

pub struct DepthResources {
    device: Rc<Device>,
    depth_image: vk::Image,
    depth_image_memory: vk::DeviceMemory,
    depth_image_view: vk::ImageView,
}

impl Drop for DepthResources {
    fn drop(&mut self) {
        self.device.destroy_image_view(self.depth_image_view);
        self.device.destroy_image(self.depth_image);
        self.device.free_memory(self.depth_image_memory);
    }
}

impl DepthResources {
    pub fn get_image_view(&self) -> vk::ImageView {
        self.depth_image_view
    }
}

pub struct DepthResourcesBuilder<'a> {
    instance: &'a Instance,
    physical_device: PhysicalDevice,
    device: Rc<Device>,
    command_buffers: &'a CommandBuffers,
    width: u32,
    height: u32,
}

impl<'a> DepthResourcesBuilder<'a> {
    pub fn new(
        instance: &'a Instance,
        physical_device: PhysicalDevice,
        device: Rc<Device>,
        command_buffers: &'a CommandBuffers,
    ) -> Self {
        DepthResourcesBuilder {
            instance,
            physical_device,
            device,
            command_buffers,
            width: 0,
            height: 0,
        }
    }

    pub fn with_width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    pub fn with_height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    pub fn build(self) -> Result<DepthResources, VulkanError> {
        let depth_format = self
            .instance
            .find_depth_format(self.physical_device)
            .ok_or_else(|| {
                VulkanError::DepthResourcesCreationError(String::from("Cannot find depth format"))
            })?;
        let (depth_image, depth_image_memory) = self.create_image(
            self.width,
            self.height,
            depth_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        let depth_image_view =
            self.create_image_view(depth_image, depth_format, vk::ImageAspectFlags::DEPTH)?;

        self.transition_image_layout(
            depth_image,
            depth_format,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        )?;

        Ok(DepthResources {
            device: self.device,
            depth_image,
            depth_image_memory,
            depth_image_view,
        })
    }

    fn create_image(
        &self,
        width: u32,
        height: u32,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<(vk::Image, vk::DeviceMemory), VulkanError> {
        let image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(
                vk::Extent3D::builder()
                    .width(width)
                    .height(height)
                    .depth(1)
                    .build(),
            )
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(tiling)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(usage)
            .samples(vk::SampleCountFlags::TYPE_1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();

        let image = self.device.create_image(&image_info)?;
        let mem_requirements = self.device.get_image_memory_requirements(image);

        let memory_type_index = self
            .instance
            .find_memory_type(
                self.physical_device,
                mem_requirements.memory_type_bits,
                properties,
            )
            .ok_or_else(|| {
                VulkanError::DepthResourcesCreationError(String::from("Cannot find a memory type"))
            })?;

        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(memory_type_index)
            .build();
        let image_memory = self.device.allocate_memory(&alloc_info)?;

        self.device.bind_image_memory(image, image_memory)?;

        Ok((image, image_memory))
    }

    fn create_image_view(
        &self,
        image: vk::Image,
        format: vk::Format,
        aspect_flags: vk::ImageAspectFlags,
    ) -> Result<vk::ImageView, VulkanError> {
        let view_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(aspect_flags)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
            )
            .build();

        self.device.create_image_view(&view_info)
    }

    fn transition_image_layout(
        &self,
        image: vk::Image,
        format: vk::Format,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) -> Result<(), VulkanError> {
        let command_buffer = self.command_buffers.begin_single_time_commands()?;

        let aspect_mask = if new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
            if format == vk::Format::D32_SFLOAT_S8_UINT || format == vk::Format::D24_UNORM_S8_UINT {
                vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL
            } else {
                vk::ImageAspectFlags::DEPTH
            }
        } else {
            vk::ImageAspectFlags::COLOR
        };

        let (src_access_mask, dst_access_mask, src_stage, dst_stage) = if old_layout
            == vk::ImageLayout::UNDEFINED
            && (new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
                || new_layout == vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
        {
            (
                vk::AccessFlags::empty(),
                vk::AccessFlags::TRANSFER_WRITE,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
            )
        } else if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
            && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        {
            (
                vk::AccessFlags::TRANSFER_WRITE,
                vk::AccessFlags::SHADER_READ,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
            )
        } else if old_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        {
            (
                vk::AccessFlags::empty(),
                vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
        } else {
            return Err(VulkanError::DepthResourcesCreationError(String::from(
                "unsupported layout transition",
            )));
        };

        let barrier = vk::ImageMemoryBarrier::builder()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(aspect_mask)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
            )
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask)
            .build();

        self.device.cmd_pipeline_barrier(
            command_buffer,
            src_stage,
            dst_stage,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );

        self.command_buffers
            .end_single_time_commands(command_buffer)
    }
}
