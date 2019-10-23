use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum VulkanError {
    InstanceCreationError(String),
    DebugCreationError(String),
    InstanceError(String),
    SurfaceError(String),
    PhysicalDeviceCreationError(String),
    DeviceError(String),
    SwapchainCreationError(String),
    RenderPassCreationError(String),
    DepthResourcesCreationError(String),
    SwapchainError(String),
    ShaderCreationError(String),
    VertexBufferCreationError(String),
    ImageCreationError(String),
    TextureCreationError(String),
    GraphicsPipelineCreationError(String),
    RayTracingError(String),
}

impl Display for VulkanError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Vulkan Error: {:?}", self)
    }
}
