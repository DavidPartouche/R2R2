use std::ffi::CStr;

#[derive(Debug, PartialEq)]
pub enum DeviceExtensions {
    ExtDescriptorIndexing,
    KhrSwapchain,
    NvRayTracing,
    NotImplemented,
}

impl From<&str> for DeviceExtensions {
    fn from(name: &str) -> Self {
        match name {
            "VK_EXT_descriptor_indexing" => DeviceExtensions::ExtDescriptorIndexing,
            "VK_KHR_swapchain" => DeviceExtensions::KhrSwapchain,
            "VK_NV_ray_tracing" => DeviceExtensions::NvRayTracing,
            _ => DeviceExtensions::NotImplemented,
        }
    }
}

impl DeviceExtensions {
    pub fn name(&self) -> &'static CStr {
        match *self {
            DeviceExtensions::ExtDescriptorIndexing => {
                CStr::from_bytes_with_nul(b"VK_EXT_descriptor_indexing\0").unwrap()
            }
            DeviceExtensions::KhrSwapchain => {
                CStr::from_bytes_with_nul(b"VK_KHR_swapchain\0").unwrap()
            }
            DeviceExtensions::NvRayTracing => {
                CStr::from_bytes_with_nul(b"VK_NV_ray_tracing\0").unwrap()
            }
            DeviceExtensions::NotImplemented => {
                CStr::from_bytes_with_nul(b"NotImplemented\0").unwrap()
            }
        }
    }
}
