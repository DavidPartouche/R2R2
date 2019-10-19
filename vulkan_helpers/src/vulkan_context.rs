use std::mem;
use std::os::raw::c_void;
use std::ptr::null;
use std::rc::Rc;

use ash::vk;

use crate::buffer::{Buffer, BufferBuilder, BufferType};
use crate::command_buffers::{CommandBuffers, CommandBuffersBuilder};
use crate::depth_resources::{DepthResources, DepthResourcesBuilder};
use crate::descriptor_pool::{DescriptorPool, DescriptorPoolBuilder};
use crate::device::{VulkanDevice, VulkanDeviceBuilder};
use crate::errors::VulkanError;
use crate::extensions::DeviceExtensions;
use crate::frame_buffer::{FrameBuffers, FrameBuffersBuilder};
use crate::image_views::{ImageViews, ImageViewsBuilder};
use crate::images::Image;
use crate::instance::{VulkanInstance, VulkanInstanceBuilder};
use crate::material::Material;
use crate::physical_device::{PhysicalDevice, PhysicalDeviceBuilder};
use crate::present_mode::{PresentMode, PresentModeBuilder};
use crate::queue_family::{QueueFamily, QueueFamilyBuilder};
use crate::render_pass::{RenderPass, RenderPassBuilder};
use crate::surface::{Surface, SurfaceBuilder};
use crate::surface_format::{SurfaceFormat, SurfaceFormatBuilder};
use crate::swapchain::{Swapchain, SwapchainBuilder};
use crate::texture::{Texture, TextureBuilder};
use crate::vertex::Vertex;

pub struct VulkanContext {
    frame_buffers: FrameBuffers,
    depth_resources: DepthResources,
    image_views: ImageViews,
    pub(crate) render_pass: RenderPass,
    swapchain: Swapchain,
    pub(crate) descriptor_pool: Rc<DescriptorPool>,
    pub(crate) command_buffers: CommandBuffers,
    pub(crate) device: Rc<VulkanDevice>,
    present_mode: PresentMode,
    surface_format: SurfaceFormat,
    queue_family: QueueFamily,
    pub(crate) physical_device: PhysicalDevice,
    surface: Surface,
    pub(crate) instance: Rc<VulkanInstance>,
    frame_index: usize,
    frames_count: usize,
    image_index: usize,
    pub(crate) width: u32,
    pub(crate) height: u32,
    clear_value: [f32; 4],
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        self.device.queue_wait_idle().unwrap();
    }
}

impl VulkanContext {
    pub fn set_clear_value(&mut self, clear_value: [f32; 4]) {
        self.clear_value = clear_value;
    }

    pub fn create_vertex_buffer(&self, vertices: &[Vertex]) -> Result<Buffer, VulkanError> {
        let size = (mem::size_of::<Vertex>() * vertices.len()) as vk::DeviceSize;
        let vertices = vertices.as_ptr() as *const c_void;
        self.create_buffer(BufferType::Vertex, size, vertices)
    }

    pub fn create_index_buffer(&self, indices: &[u32]) -> Result<Buffer, VulkanError> {
        let size = (mem::size_of::<u32>() * indices.len()) as vk::DeviceSize;
        let indices = indices.as_ptr() as *const c_void;
        self.create_buffer(BufferType::Index, size, indices)
    }

    pub fn create_material_buffer(&self, materials: &[Material]) -> Result<Buffer, VulkanError> {
        let size = (mem::size_of::<Material>() * materials.len()) as vk::DeviceSize;
        let materials = materials.as_ptr() as *const c_void;

        let mat_buffer = BufferBuilder::new(self)
            .with_type(BufferType::Storage)
            .with_size(size)
            .build()?;
        mat_buffer.copy_data(materials)?;

        Ok(mat_buffer)
    }

    pub fn create_texture_images(&self, images: &Vec<Image>) -> Result<Vec<Texture>, VulkanError> {
        let mut textures = vec![];

        if images.is_empty() {
            let image = Image {
                pixels: vec![255, 0, 255, 255],
                tex_width: 1,
                tex_height: 1,
                tex_channels: 1,
            };

            let texture = TextureBuilder::new(self).with_image(&image).build()?;
            textures.push(texture);
        }

        for image in images {
            let texture = TextureBuilder::new(self).with_image(image).build()?;
            textures.push(texture);
        }

        Ok(textures)
    }

    pub fn create_buffer(
        &self,
        ty: BufferType,
        size: vk::DeviceSize,
        data: *const c_void,
    ) -> Result<Buffer, VulkanError> {
        let staging_buffer = BufferBuilder::new(self)
            .with_type(BufferType::Staging)
            .with_size(size)
            .build()?;

        staging_buffer.copy_data(data)?;

        let buffer = BufferBuilder::new(self)
            .with_type(ty)
            .with_size(size)
            .build()?;

        self.command_buffers
            .copy_buffer(staging_buffer.get(), buffer.get(), size)?;

        Ok(buffer)
    }

    pub fn frame_begin(&mut self) -> Result<(), VulkanError> {
        self.command_buffers.wait_for_fence(self.frame_index)?;

        self.image_index = self.swapchain.acquire_next_image(
            self.command_buffers
                .get_present_complete_semaphore(self.frame_index),
        )?;

        self.command_buffers.begin_command_buffer(self.frame_index)
    }

    pub fn frame_end(&self) -> Result<(), VulkanError> {
        self.command_buffers.end_command_buffer(self.frame_index)?;
        self.command_buffers.reset_fence(self.frame_index)?;
        self.command_buffers.queue_submit(self.frame_index)
    }

    pub fn frame_present(&mut self) -> Result<(), VulkanError> {
        self.swapchain.queue_present(
            self.command_buffers
                .get_render_complete_semaphore(self.frame_index),
            self.image_index as u32,
        )?;
        self.frame_index = (self.frame_index + 1) % self.frames_count;
        Ok(())
    }

    pub fn begin_render_pass(&self) {
        let clear_color = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: self.clear_value,
            },
        };
        let clear_depth = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue::builder()
                .depth(1.0)
                .stencil(0)
                .build(),
        };
        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass.get())
            .framebuffer(self.frame_buffers.get(self.image_index))
            .render_area(
                vk::Rect2D::builder()
                    .extent(
                        vk::Extent2D::builder()
                            .width(self.width)
                            .height(self.height)
                            .build(),
                    )
                    .build(),
            )
            .clear_values(&[clear_color, clear_depth])
            .build();

        self.device
            .cmd_begin_render_pass(self.command_buffers.get(self.frame_index), &info);
    }
    pub fn end_render_pass(&self) {
        self.device
            .cmd_end_render_pass(self.command_buffers.get(self.frame_index));
    }

    pub fn get_current_command_buffer(&self) -> vk::CommandBuffer {
        self.command_buffers.get(self.frame_index)
    }

    pub fn begin_single_time_commands(&self) -> Result<vk::CommandBuffer, VulkanError> {
        self.command_buffers
            .begin_single_time_commands(self.frame_index)
    }

    pub fn end_single_time_commands(
        &self,
        command_buffer: vk::CommandBuffer,
    ) -> Result<(), VulkanError> {
        self.command_buffers
            .end_single_time_commands(command_buffer, self.frame_index)
    }
}

pub struct VulkanContextBuilder {
    debug: bool,
    hwnd: *const c_void,
    width: u32,
    height: u32,
    extensions: Vec<DeviceExtensions>,
    frames_count: usize,
}

impl Default for VulkanContextBuilder {
    fn default() -> Self {
        VulkanContextBuilder {
            debug: false,
            hwnd: null(),
            width: 0,
            height: 0,
            extensions: vec![],
            frames_count: 2,
        }
    }
}

impl VulkanContextBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_debug_enabled(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn with_hwnd(mut self, hwnd: *const c_void) -> Self {
        self.hwnd = hwnd;
        self
    }

    pub fn with_width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    pub fn with_height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    pub fn with_extensions(mut self, extensions: Vec<DeviceExtensions>) -> Self {
        self.extensions = extensions;
        self
    }

    pub fn with_frames_count(mut self, frames_count: usize) -> Self {
        self.frames_count = frames_count;
        self
    }

    pub fn build(self) -> Result<VulkanContext, VulkanError> {
        let instance = Rc::new(self.create_instance()?);
        let surface = self.create_surface(&instance)?;
        let physical_device = self.get_physical_device(&instance, &surface)?;
        let queue_family = self.get_queue_family(&instance, &surface, physical_device)?;
        let surface_format = self.find_surface_format(&surface, physical_device)?;
        let present_mode = self.get_present_mode(&surface, physical_device)?;
        let device = Rc::new(self.create_logical_device(
            Rc::clone(&instance),
            physical_device,
            queue_family,
        )?);
        let command_buffers = self.create_command_buffers(queue_family, Rc::clone(&device))?;
        let descriptor_pool = Rc::new(self.create_descriptor_pool(Rc::clone(&device))?);
        let swapchain = self.create_swapchain(
            Rc::clone(&device),
            &surface,
            physical_device,
            surface_format,
            present_mode,
        )?;
        let render_pass = self.create_render_pass(
            &instance,
            physical_device,
            Rc::clone(&device),
            surface_format,
        )?;
        let image_views =
            self.create_image_views(Rc::clone(&device), surface_format, &swapchain)?;
        let depth_resources = self.create_depth_resources(
            &instance,
            physical_device,
            Rc::clone(&device),
            &command_buffers,
        )?;
        let frame_buffers = self.create_frame_buffers(
            Rc::clone(&device),
            &render_pass,
            &image_views,
            &depth_resources,
        )?;

        Ok(VulkanContext {
            instance,
            surface,
            physical_device,
            queue_family,
            surface_format,
            present_mode,
            device,
            command_buffers,
            descriptor_pool,
            swapchain,
            render_pass,
            image_views,
            depth_resources,
            frame_buffers,
            frame_index: 0,
            frames_count: self.frames_count,
            image_index: 0,
            width: self.width,
            height: self.height,
            clear_value: [1.0, 1.0, 1.0, 1.0],
        })
    }

    fn create_instance(&self) -> Result<VulkanInstance, VulkanError> {
        VulkanInstanceBuilder::new()
            .with_debug_enabled(self.debug)
            .build()
    }

    fn create_surface(&self, instance: &VulkanInstance) -> Result<Surface, VulkanError> {
        SurfaceBuilder::new(instance).with_hwnd(self.hwnd).build()
    }

    fn get_physical_device(
        &self,
        instance: &VulkanInstance,
        surface: &Surface,
    ) -> Result<PhysicalDevice, VulkanError> {
        PhysicalDeviceBuilder::new(instance, surface)
            .with_extensions(&self.extensions)
            .build()
    }

    fn get_queue_family(
        &self,
        instance: &VulkanInstance,
        surface: &Surface,
        physical_device: PhysicalDevice,
    ) -> Result<QueueFamily, VulkanError> {
        QueueFamilyBuilder::new(instance, surface, physical_device).build()
    }

    fn find_surface_format(
        &self,
        surface: &Surface,
        physical_device: PhysicalDevice,
    ) -> Result<SurfaceFormat, VulkanError> {
        SurfaceFormatBuilder::new(surface, physical_device).build()
    }

    fn get_present_mode(
        &self,
        surface: &Surface,
        physical_device: PhysicalDevice,
    ) -> Result<PresentMode, VulkanError> {
        PresentModeBuilder::new(surface, physical_device).build()
    }

    fn create_logical_device(
        &self,
        instance: Rc<VulkanInstance>,
        physical_device: PhysicalDevice,
        queue_family: QueueFamily,
    ) -> Result<VulkanDevice, VulkanError> {
        VulkanDeviceBuilder::new(instance, physical_device, queue_family)
            .with_extensions(&self.extensions)
            .build()
    }

    fn create_command_buffers(
        &self,
        queue_family: QueueFamily,
        device: Rc<VulkanDevice>,
    ) -> Result<CommandBuffers, VulkanError> {
        CommandBuffersBuilder::new(queue_family, device)
            .with_buffer_count(self.frames_count)
            .build()
    }

    fn create_descriptor_pool(
        &self,
        device: Rc<VulkanDevice>,
    ) -> Result<DescriptorPool, VulkanError> {
        DescriptorPoolBuilder::new(device).build()
    }

    fn create_swapchain(
        &self,
        device: Rc<VulkanDevice>,
        surface: &Surface,
        physical_device: PhysicalDevice,
        surface_format: SurfaceFormat,
        present_mode: PresentMode,
    ) -> Result<Swapchain, VulkanError> {
        SwapchainBuilder::new(
            device,
            surface,
            physical_device,
            surface_format,
            present_mode,
        )
        .with_width(self.width)
        .with_height(self.height)
        .build()
    }

    fn create_render_pass(
        &self,
        instance: &VulkanInstance,
        physical_device: PhysicalDevice,
        device: Rc<VulkanDevice>,
        surface_format: SurfaceFormat,
    ) -> Result<RenderPass, VulkanError> {
        RenderPassBuilder::new(instance, physical_device, device, surface_format).build()
    }

    fn create_image_views(
        &self,
        device: Rc<VulkanDevice>,
        surface_format: SurfaceFormat,
        swapchain: &Swapchain,
    ) -> Result<ImageViews, VulkanError> {
        ImageViewsBuilder::new(device, surface_format, swapchain).build()
    }

    fn create_depth_resources(
        &self,
        instance: &VulkanInstance,
        physical_device: PhysicalDevice,
        device: Rc<VulkanDevice>,
        command_buffers: &CommandBuffers,
    ) -> Result<DepthResources, VulkanError> {
        DepthResourcesBuilder::new(instance, physical_device, device, command_buffers)
            .with_width(self.width)
            .with_height(self.height)
            .build()
    }

    fn create_frame_buffers(
        &self,
        device: Rc<VulkanDevice>,
        render_pass: &RenderPass,
        image_views: &ImageViews,
        depth_resources: &DepthResources,
    ) -> Result<FrameBuffers, VulkanError> {
        FrameBuffersBuilder::new(device, render_pass, image_views, depth_resources)
            .with_width(self.width)
            .with_height(self.height)
            .build()
    }
}
