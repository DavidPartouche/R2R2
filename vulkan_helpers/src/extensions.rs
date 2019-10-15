use std::ffi::CStr;

#[derive(Debug, PartialEq)]
pub enum ExtensionProperties {
    KhrSwapchain,
    NvRayTracing,
    NotImplemented,
}

impl From<&str> for ExtensionProperties {
    fn from(name: &str) -> Self {
        match name {
            "VK_KHR_swapchain" => ExtensionProperties::KhrSwapchain,
            "VK_NV_ray_tracing" => ExtensionProperties::NvRayTracing,
            _ => ExtensionProperties::NotImplemented,
        }
    }
}

impl ExtensionProperties {
    pub fn name(&self) -> &'static CStr {
        match *self {
            ExtensionProperties::KhrSwapchain => {
                CStr::from_bytes_with_nul(b"VK_KHR_swapchain\0").unwrap()
            }
            ExtensionProperties::NvRayTracing => {
                CStr::from_bytes_with_nul(b"VK_NV_ray_tracing\0").unwrap()
            }
            ExtensionProperties::NotImplemented => {
                CStr::from_bytes_with_nul(b"NotImplemented\0").unwrap()
            }
        }
    }
}
