use std::rc::Rc;

use ash::vk;

use crate::buffer::Buffer;
use crate::descriptor_pool::DescriptorPool;
use crate::descriptor_set_layout::DescriptorSetLayout;
use crate::device::Device;
use crate::errors::VulkanError;
use crate::texture::Texture;
use crate::vulkan_context::VulkanContext;

pub struct DescriptorSet {
    device: Rc<Device>,
    descriptor_pool: Rc<DescriptorPool>,
    descriptor_set: vk::DescriptorSet,
}

impl Drop for DescriptorSet {
    fn drop(&mut self) {
        self.device
            .free_descriptor_sets(self.descriptor_pool.get(), &[self.descriptor_set]);
    }
}

impl DescriptorSet {
    pub fn get(&self) -> vk::DescriptorSet {
        self.descriptor_set
    }
}

pub struct DescriptorSetBuilder<'a> {
    context: &'a VulkanContext,
    descriptor_set_layout: &'a DescriptorSetLayout,
    uniform_buffer: Option<&'a Buffer>,
    mat_color_buffer: Option<&'a Buffer>,
    textures: Option<&'a [Texture]>,
}

impl<'a> DescriptorSetBuilder<'a> {
    pub fn new(context: &'a VulkanContext, descriptor_set_layout: &'a DescriptorSetLayout) -> Self {
        DescriptorSetBuilder {
            context,
            descriptor_set_layout,
            uniform_buffer: None,
            mat_color_buffer: None,
            textures: None,
        }
    }

    pub fn with_uniform_buffer(mut self, buffer: &'a Buffer) -> Self {
        self.uniform_buffer = Some(buffer);
        self
    }

    pub fn with_material_buffer(mut self, buffer: &'a Buffer) -> Self {
        self.mat_color_buffer = Some(buffer);
        self
    }

    pub fn with_textures(mut self, textures: &'a [Texture]) -> Self {
        self.textures = Some(textures);
        self
    }

    pub fn build(self) -> Result<DescriptorSet, VulkanError> {
        let alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.context.descriptor_pool.get())
            .set_layouts(&[self.descriptor_set_layout.get()])
            .build();

        let descriptor_set = self.context.device.allocate_descriptor_sets(&alloc_info)?[0];

        let buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(self.uniform_buffer.unwrap().get())
            .offset(0)
            .range(vk::WHOLE_SIZE)
            .build();

        let mat_color_buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(self.mat_color_buffer.unwrap().get())
            .offset(0)
            .range(vk::WHOLE_SIZE)
            .build();

        let mut image_infos = vec![];
        for texture in self.textures.unwrap() {
            let image_info = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(texture.get_image_view())
                .sampler(texture.get_sampler())
                .build();
            image_infos.push(image_info);
        }

        let uniform_wds = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_set)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .dst_binding(0)
            .buffer_info(&[buffer_info])
            .build();

        let material_wds = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_set)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .dst_binding(1)
            .buffer_info(&[mat_color_buffer_info])
            .build();

        let textures_wds = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_set)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .dst_binding(2)
            .image_info(image_infos.as_slice())
            .build();

        self.context
            .device
            .update_descriptor_sets(&[uniform_wds, material_wds, textures_wds]);

        Ok(DescriptorSet {
            device: Rc::clone(&self.context.device),
            descriptor_pool: Rc::clone(&self.context.descriptor_pool),
            descriptor_set,
        })
    }
}
