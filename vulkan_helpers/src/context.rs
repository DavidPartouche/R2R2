use std::mem;
use std::os::raw::c_void;
use std::ptr::null;
use std::rc::Rc;

use crate::command_buffers::{CommandBuffers, CommandBuffersBuilder};
use crate::descriptor_pool::{DescriptorPool, DescriptorPoolBuilder};
use crate::device::{Device, DeviceBuilder};
use crate::errors::VulkanError;
use crate::extensions::ExtensionProperties;
use crate::instance::{Instance, InstanceBuilder};
use crate::physical_device::{PhysicalDevice, PhysicalDeviceBuilder};
use crate::present_mode::{PresentMode, PresentModeBuilder};
use crate::queue_family::{QueueFamily, QueueFamilyBuilder};
use crate::surface::{Surface, SurfaceBuilder};
use crate::surface_format::{SurfaceFormat, SurfaceFormatBuilder};
use crate::swapchain_context::{SwapchainContext, SwapchainContextBuilder};

pub struct VulkanContext {
    swapchain_context: SwapchainContext,
    descriptor_pool: DescriptorPool,
    command_buffers: CommandBuffers,
    device: Rc<Device>,
    present_mode: PresentMode,
    surface_format: SurfaceFormat,
    queue_family: QueueFamily,
    physical_device: PhysicalDevice,
    surface: Surface,
    instance: Rc<Instance>,
}

impl VulkanContext {
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), VulkanError> {
        self.device.queue_wait_idle()?;
        let swapchain_context = SwapchainContextBuilder::new(
            Rc::clone(&self.device),
            &self.surface,
            self.physical_device,
            self.surface_format,
            self.present_mode,
        )
        .with_old_swapchain(self.swapchain_context.get_swapchain())
        .with_width(width)
        .with_height(height)
        .build()?;
        mem::replace(&mut self.swapchain_context, swapchain_context);
        Ok(())
    }
}

pub struct VulkanContextBuilder {
    debug: bool,
    hwnd: *const c_void,
    width: u32,
    height: u32,
    extensions: Vec<ExtensionProperties>,
}

impl Default for VulkanContextBuilder {
    fn default() -> Self {
        VulkanContextBuilder {
            debug: false,
            hwnd: null(),
            width: 0,
            height: 0,
            extensions: vec![],
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

    pub fn with_extensions(mut self, extensions: Vec<ExtensionProperties>) -> Self {
        self.extensions = extensions;
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
        let descriptor_pool = self.create_descriptor_pool(Rc::clone(&device))?;
        let swapchain_context = self.create_swapchain_context(
            Rc::clone(&device),
            &surface,
            physical_device,
            surface_format,
            present_mode,
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
            swapchain_context,
        })
    }

    fn create_instance(&self) -> Result<Instance, VulkanError> {
        InstanceBuilder::new()
            .with_debug_enabled(self.debug)
            .build()
    }

    fn create_surface(&self, instance: &Instance) -> Result<Surface, VulkanError> {
        SurfaceBuilder::new(instance).with_hwnd(self.hwnd).build()
    }

    fn get_physical_device(
        &self,
        instance: &Instance,
        surface: &Surface,
    ) -> Result<PhysicalDevice, VulkanError> {
        PhysicalDeviceBuilder::new(instance, surface)
            .with_extensions(&self.extensions)
            .build()
    }

    fn get_queue_family(
        &self,
        instance: &Instance,
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
        instance: Rc<Instance>,
        physical_device: PhysicalDevice,
        queue_family: QueueFamily,
    ) -> Result<Device, VulkanError> {
        DeviceBuilder::new(instance, physical_device, queue_family)
            .with_extensions(&self.extensions)
            .build()
    }

    fn create_command_buffers(
        &self,
        queue_family: QueueFamily,
        device: Rc<Device>,
    ) -> Result<CommandBuffers, VulkanError> {
        CommandBuffersBuilder::new(queue_family, device)
            .with_buffer_count(2)
            .build()
    }

    fn create_descriptor_pool(&self, device: Rc<Device>) -> Result<DescriptorPool, VulkanError> {
        DescriptorPoolBuilder::new(device).build()
    }

    fn create_swapchain_context(
        &self,
        device: Rc<Device>,
        surface: &Surface,
        physical_device: PhysicalDevice,
        surface_format: SurfaceFormat,
        present_mode: PresentMode,
    ) -> Result<SwapchainContext, VulkanError> {
        SwapchainContextBuilder::new(
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
}
