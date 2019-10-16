use std::rc::Rc;

use ash::vk;

use crate::device::Device;
use crate::errors::VulkanError;
use crate::instance::Instance;
use crate::physical_device::PhysicalDevice;
use crate::surface_format::SurfaceFormat;

pub struct RenderPass {
    device: Rc<Device>,
    render_pass: vk::RenderPass,
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        self.device.destroy_render_pass(self.render_pass);
    }
}

pub struct RenderPassBuilder<'a> {
    instance: &'a Instance,
    physical_device: PhysicalDevice,
    device: Rc<Device>,
    surface_format: SurfaceFormat,
}

impl<'a> RenderPassBuilder<'a> {
    pub fn new(
        instance: &'a Instance,
        physical_device: PhysicalDevice,
        device: Rc<Device>,
        surface_format: SurfaceFormat,
    ) -> Self {
        RenderPassBuilder {
            instance,
            physical_device,
            device,
            surface_format,
        }
    }

    pub fn build(self) -> Result<RenderPass, VulkanError> {
        let color_attachment = vk::AttachmentDescription::builder()
            .format(self.surface_format.format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build();

        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        let format = self
            .instance
            .find_depth_format(self.physical_device)
            .ok_or_else(|| {
                VulkanError::RenderPassCreationError(String::from("Cannot find depth format"))
            })?;

        let depth_attachment = vk::AttachmentDescription::builder()
            .format(format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        let depth_attachment_ref = vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&[color_attachment_ref])
            .depth_stencil_attachment(&depth_attachment_ref)
            .build();

        let dependencies = [
            vk::SubpassDependency::builder()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::BOTTOM_OF_PIPE)
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .src_access_mask(vk::AccessFlags::MEMORY_READ)
                .dst_access_mask(
                    vk::AccessFlags::COLOR_ATTACHMENT_READ
                        | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                )
                .dependency_flags(vk::DependencyFlags::BY_REGION)
                .build(),
            vk::SubpassDependency::builder()
                .src_subpass(0)
                .dst_subpass(1)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_stage_mask(vk::PipelineStageFlags::BOTTOM_OF_PIPE)
                .src_access_mask(
                    vk::AccessFlags::COLOR_ATTACHMENT_READ
                        | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                )
                .dst_access_mask(vk::AccessFlags::MEMORY_READ)
                .dependency_flags(vk::DependencyFlags::BY_REGION)
                .build(),
        ];

        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(&[color_attachment, depth_attachment])
            .subpasses(&[subpass, subpass])
            .dependencies(&dependencies)
            .build();

        let render_pass = self.device.create_render_pass(&render_pass_info)?;

        Ok(RenderPass {
            device: self.device,
            render_pass,
        })
    }
}
