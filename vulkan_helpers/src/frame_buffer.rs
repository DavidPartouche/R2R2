use std::rc::Rc;

use ash::vk;

use crate::depth_resources::DepthResources;
use crate::device::VulkanDevice;
use crate::errors::VulkanError;
use crate::image_views::ImageViews;
use crate::render_pass::RenderPass;

pub struct FrameBuffers {
    device: Rc<VulkanDevice>,
    frame_buffers: Vec<vk::Framebuffer>,
}

impl Drop for FrameBuffers {
    fn drop(&mut self) {
        for frame_buffer in self.frame_buffers.iter() {
            self.device.destroy_frame_buffer(*frame_buffer);
        }
    }
}

impl FrameBuffers {
    pub fn get(&self, index: usize) -> vk::Framebuffer {
        self.frame_buffers[index]
    }
}

pub struct FrameBuffersBuilder<'a> {
    device: Rc<VulkanDevice>,
    render_pass: &'a RenderPass,
    image_views: &'a ImageViews,
    depth_resources: &'a DepthResources,
    width: u32,
    height: u32,
}

impl<'a> FrameBuffersBuilder<'a> {
    pub fn new(
        device: Rc<VulkanDevice>,
        render_pass: &'a RenderPass,
        image_views: &'a ImageViews,
        depth_resources: &'a DepthResources,
    ) -> Self {
        FrameBuffersBuilder {
            device,
            render_pass,
            image_views,
            depth_resources,
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

    pub fn build(self) -> Result<FrameBuffers, VulkanError> {
        let mut frame_buffers = vec![];

        for image_view in self.image_views.get_image_views() {
            let framebuffer_info = vk::FramebufferCreateInfo::builder()
                .render_pass(self.render_pass.get())
                .attachments(&[*image_view, self.depth_resources.get_image_view()])
                .width(self.width)
                .height(self.height)
                .layers(1)
                .build();
            frame_buffers.push(self.device.create_frame_buffer(&framebuffer_info)?);
        }

        Ok(FrameBuffers {
            device: self.device,
            frame_buffers,
        })
    }
}
