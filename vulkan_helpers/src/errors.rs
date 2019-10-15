use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum VulkanError {
    InstanceCreationError(String),
    DebugCreationError(String),
    InstanceError(String),
    SurfaceError(String),
    PhysicalDeviceCreationError(String),
    QueueFamilyCreationError(String),
    DeviceError(String),
    SwapchainCreationError(String),
    //    DeviceCreationError(String),
    //    RenderPassCreationError(String),
    //    ShaderCreationError(String),
    //    PipelineCreationError(String),
    //    FramebufferCreationError(String),
    //    SemaphoreCreationError(String),
    //    FenceCreationError(String),
    //    VertexBufferCreationError(String),
    //    DescriptorSetLayoutCreationError(String),
    //    TextureImageCreationError(String),
    //    SwapchainError(String),
    //    FenceWaitError(String),
}

impl Display for VulkanError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Vulkan Error: {:?}", self)
    }
}
