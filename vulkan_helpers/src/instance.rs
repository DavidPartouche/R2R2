use crate::errors::VulkanError;
use ash::extensions::{ext, khr};
use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk;
use std::ffi::CStr;
use std::os::raw::c_void;
use std::ptr::null;

pub struct VulkanInstance {
    entry: ash::Entry,
    instance: ash::Instance,
    debug_utils: Option<ash::extensions::ext::DebugUtils>,
    messenger: Option<vk::DebugUtilsMessengerEXT>,
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        unsafe {
            if let Some(debug_utils) = &self.debug_utils {
                debug_utils.destroy_debug_utils_messenger(self.messenger.unwrap(), None);
            }
            self.instance.destroy_instance(None);
        }
    }
}

impl VulkanInstance {
    pub fn create_win_32_surface(
        &self,
        hwnd: vk::HWND,
    ) -> Result<(khr::Surface, vk::SurfaceKHR), VulkanError> {
        let hinstance = null() as vk::HINSTANCE;

        let create_info = vk::Win32SurfaceCreateInfoKHR::builder()
            .hinstance(hinstance)
            .hwnd(hwnd)
            .build();

        let surface_loader = khr::Surface::new(&self.entry, &self.instance);

        let win32_surface_loader = khr::Win32Surface::new(&self.entry, &self.instance);

        let surface = unsafe { win32_surface_loader.create_win32_surface(&create_info, None) }
            .map_err(|err| VulkanError::InstanceError(err.to_string()))?;

        Ok((surface_loader, surface))
    }

    unsafe extern "system" fn vulkan_debug_callback(
        severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        ty: vk::DebugUtilsMessageTypeFlagsEXT,
        callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _: *mut c_void,
    ) -> u32 {
        let message = CStr::from_ptr((*callback_data).p_message);

        let message = if ty.contains(vk::DebugUtilsMessageTypeFlagsEXT::GENERAL) {
            format!("General Layer: {:?}", message)
        } else if ty.contains(vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION) {
            format!("Validation layer: {:?}", message)
        } else {
            format!("Performance Layer: {:?}", message)
        };

        if severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE) {
            log::trace!("{}", message);
        } else if severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::INFO) {
            log::info!("{}", message);
        } else if severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::WARNING) {
            log::warn!("{}", message);
        } else if severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::ERROR) {
            log::error!("{}", message);
        }

        vk::FALSE
    }
}

pub struct VulkanInstanceBuilder {
    debug: bool,
}

impl VulkanInstanceBuilder {
    pub fn new() -> Self {
        VulkanInstanceBuilder { debug: false }
    }

    pub fn with_debug_enabled(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn build(self) -> Result<VulkanInstance, VulkanError> {
        let name = CStr::from_bytes_with_nul(b"R2R2\0").unwrap();
        let version = ash::vk_make_version!(0, 1, 0);
        let api_version = ash::vk_make_version!(1, 1, 0);

        let application_info = vk::ApplicationInfo::builder()
            .application_name(name)
            .application_version(version)
            .engine_name(name)
            .engine_version(version)
            .api_version(api_version)
            .build();

        let mut layers = vec![];
        let mut extensions = vec![
            khr::Surface::name().as_ptr(),
            khr::Win32Surface::name().as_ptr(),
        ];

        if self.debug {
            let debug_layer = CStr::from_bytes_with_nul(b"VK_LAYER_KHRONOS_validation\0").unwrap();
            layers.push(debug_layer.as_ptr());
            extensions.push(ext::DebugUtils::name().as_ptr())
        }

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_layer_names(layers.as_slice())
            .enabled_extension_names(extensions.as_slice())
            .build();

        let entry =
            ash::Entry::new().map_err(|err| VulkanError::InstanceCreationError(err.to_string()))?;
        let instance = unsafe { entry.create_instance(&create_info, None) }
            .map_err(|err| VulkanError::InstanceCreationError(err.to_string()))?;

        let (debug_utils, messenger) = if self.debug {
            let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
                .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
                .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
                .pfn_user_callback(Some(VulkanInstance::vulkan_debug_callback))
                .build();

            let debug_utils = Some(ext::DebugUtils::new(&entry, &instance));
            let messenger = Some(
                unsafe {
                    debug_utils
                        .as_ref()
                        .unwrap()
                        .create_debug_utils_messenger(&debug_info, None)
                }
                .map_err(|err| VulkanError::DebugCreationError(err.to_string()))?,
            );
            (debug_utils, messenger)
        } else {
            (None, None)
        };

        Ok(VulkanInstance {
            entry,
            instance,
            debug_utils,
            messenger,
        })
    }
}
