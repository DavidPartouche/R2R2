use std::fs::File;
use std::path::Path;
use std::rc::Rc;

use ash::util::read_spv;
use ash::vk;

use crate::device::Device;
use crate::errors::VulkanError;

pub struct ShaderModule {
    device: Rc<Device>,
    shader_module: vk::ShaderModule,
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        self.device.destroy_shader_module(self.shader_module);
    }
}

impl ShaderModule {
    pub fn get(&self) -> vk::ShaderModule {
        self.shader_module
    }
}

pub struct ShaderModuleBuilder<'a> {
    device: Rc<Device>,
    path: Option<&'a Path>,
}

impl<'a> ShaderModuleBuilder<'a> {
    pub fn new(device: Rc<Device>) -> Self {
        ShaderModuleBuilder { device, path: None }
    }

    pub fn with_path(mut self, path: &'a Path) -> Self {
        self.path = Some(path);
        self
    }

    pub fn build(self) -> Result<ShaderModule, VulkanError> {
        let shader_path = self
            .path
            .ok_or(VulkanError::ShaderCreationError(String::from(
                "Path to the shader not provided",
            )))?;
        let mut file = File::open(shader_path)
            .map_err(|err| VulkanError::ShaderCreationError(err.to_string()))?;
        let shader =
            read_spv(&mut file).map_err(|err| VulkanError::ShaderCreationError(err.to_string()))?;

        let create_info = vk::ShaderModuleCreateInfo::builder().code(&shader).build();
        let shader_module = self.device.create_shader_module(&create_info)?;

        Ok(ShaderModule {
            device: self.device,
            shader_module,
        })
    }
}
