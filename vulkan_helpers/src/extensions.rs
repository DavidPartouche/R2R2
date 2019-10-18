use std::ffi::CStr;

pub enum InstanceExtensions {
    KhrGetPhysicalDeviceProperties2,
}

impl InstanceExtensions {
    pub fn name(&self) -> &'static CStr {
        match *self {
            InstanceExtensions::KhrGetPhysicalDeviceProperties2 => {
                CStr::from_bytes_with_nul(b"VK_KHR_get_physical_device_properties2\0").unwrap()
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum DeviceExtensions {
    ExtDescriptorIndexing,
    KhrMaintenance3,
    KhrSwapchain,
    NvRayTracing,
    NotImplemented,
}

impl From<&str> for DeviceExtensions {
    fn from(name: &str) -> Self {
        match name {
            "VK_EXT_descriptor_indexing" => DeviceExtensions::ExtDescriptorIndexing,
            "VK_KHR_maintenance3" => DeviceExtensions::KhrMaintenance3,
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
            DeviceExtensions::KhrMaintenance3 => {
                CStr::from_bytes_with_nul(b"VK_KHR_maintenance3\0").unwrap()
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
