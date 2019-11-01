use std::rc::Rc;

use ash::vk;
use vulkan_bootstrap::device::VulkanDevice;
use vulkan_bootstrap::errors::VulkanError;
use vulkan_bootstrap::vulkan_context::VulkanContext;

use crate::geometry_instance::GeometryInstance;

pub struct DescriptorSet {
    device: Rc<VulkanDevice>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_set: vk::DescriptorSet,
}

impl DescriptorSet {
    pub fn get(&self) -> vk::DescriptorSet {
        self.descriptor_set
    }

    pub fn get_layout(&self) -> vk::DescriptorSetLayout {
        self.descriptor_set_layout
    }

    pub fn update_render_target(
        &mut self,
        acceleration_structure: vk::AccelerationStructureNV,
        target: vk::ImageView,
        camera_buffer: vk::Buffer,
        geometry_instance: &GeometryInstance,
    ) {
        let mut wds = vec![];

        let mut as_info = vk::WriteDescriptorSetAccelerationStructureNV::builder()
            .acceleration_structures(&[acceleration_structure])
            .build();

        let mut as_wds = vk::WriteDescriptorSet::builder()
            .dst_set(self.descriptor_set)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_NV)
            .dst_binding(0)
            .push_next(&mut as_info)
            .build();
        as_wds.descriptor_count = 1;
        wds.push(as_wds);

        let output_image_info = vk::DescriptorImageInfo::builder()
            .sampler(vk::Sampler::null())
            .image_layout(vk::ImageLayout::GENERAL)
            .image_view(target)
            .build();

        let output_image_wds = vk::WriteDescriptorSet::builder()
            .dst_set(self.descriptor_set)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
            .dst_binding(1)
            .image_info(&[output_image_info])
            .build();
        wds.push(output_image_wds);

        let cam_info = vk::DescriptorBufferInfo::builder()
            .buffer(camera_buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE)
            .build();

        let cam_wds = vk::WriteDescriptorSet::builder()
            .dst_set(self.descriptor_set)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .dst_binding(2)
            .buffer_info(&[cam_info])
            .build();
        wds.push(cam_wds);

        let vertex_info = vk::DescriptorBufferInfo::builder()
            .buffer(geometry_instance.vertex_buffer.get())
            .offset(0)
            .range(vk::WHOLE_SIZE)
            .build();

        let vertex_wds = vk::WriteDescriptorSet::builder()
            .dst_set(self.descriptor_set)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .dst_binding(3)
            .buffer_info(&[vertex_info])
            .build();
        wds.push(vertex_wds);

        let index_info = vk::DescriptorBufferInfo::builder()
            .buffer(geometry_instance.index_buffer.get())
            .offset(0)
            .range(vk::WHOLE_SIZE)
            .build();

        let index_wds = vk::WriteDescriptorSet::builder()
            .dst_set(self.descriptor_set)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .dst_binding(4)
            .buffer_info(&[index_info])
            .build();
        wds.push(index_wds);

        let mat_info = vk::DescriptorBufferInfo::builder()
            .buffer(geometry_instance.material_buffer.get())
            .offset(0)
            .range(vk::WHOLE_SIZE)
            .build();

        let mat_wds = vk::WriteDescriptorSet::builder()
            .dst_set(self.descriptor_set)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .dst_binding(5)
            .buffer_info(&[mat_info])
            .build();
        wds.push(mat_wds);

        let mut image_infos = vec![];
        for texture in geometry_instance.textures.iter() {
            let image_info = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(texture.get_image_view())
                .sampler(texture.get_sampler())
                .build();
            image_infos.push(image_info);
        }

        let textures_wds = vk::WriteDescriptorSet::builder()
            .dst_set(self.descriptor_set)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .dst_binding(6)
            .image_info(&image_infos)
            .build();
        wds.push(textures_wds);

        self.device.update_descriptor_sets(&wds);
    }
}

impl Drop for DescriptorSet {
    fn drop(&mut self) {
        self.device
            .destroy_descriptor_set_layout(self.descriptor_set_layout);
        self.device.destroy_descriptor_pool(self.descriptor_pool);
    }
}

pub struct DescriptorSetBuilder<'a> {
    context: &'a VulkanContext,
    geometry_instance: &'a GeometryInstance,
}

impl<'a> DescriptorSetBuilder<'a> {
    pub fn new(context: &'a VulkanContext, geometry_instance: &'a GeometryInstance) -> Self {
        DescriptorSetBuilder {
            context,
            geometry_instance,
        }
    }

    pub fn build(self) -> Result<DescriptorSet, VulkanError> {
        let command_buffer = self.context.begin_single_time_commands()?;

        self.cmd_pipeline_barrier(command_buffer, self.geometry_instance.vertex_buffer.get());
        self.cmd_pipeline_barrier(command_buffer, self.geometry_instance.index_buffer.get());

        self.context.end_single_time_commands(command_buffer)?;

        let mut bindings = vec![];
        bindings.push(self.add_binding(
            0,
            1,
            vk::DescriptorType::ACCELERATION_STRUCTURE_NV,
            vk::ShaderStageFlags::RAYGEN_NV,
        ));
        bindings.push(self.add_binding(
            1,
            1,
            vk::DescriptorType::STORAGE_IMAGE,
            vk::ShaderStageFlags::RAYGEN_NV,
        ));
        bindings.push(self.add_binding(
            2,
            1,
            vk::DescriptorType::UNIFORM_BUFFER,
            vk::ShaderStageFlags::RAYGEN_NV,
        ));
        bindings.push(self.add_binding(
            3,
            1,
            vk::DescriptorType::STORAGE_BUFFER,
            vk::ShaderStageFlags::CLOSEST_HIT_NV,
        ));
        bindings.push(self.add_binding(
            4,
            1,
            vk::DescriptorType::STORAGE_BUFFER,
            vk::ShaderStageFlags::CLOSEST_HIT_NV,
        ));
        bindings.push(self.add_binding(
            5,
            1,
            vk::DescriptorType::STORAGE_BUFFER,
            vk::ShaderStageFlags::CLOSEST_HIT_NV,
        ));
        bindings.push(self.add_binding(
            6,
            self.geometry_instance.textures.len() as u32,
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            vk::ShaderStageFlags::CLOSEST_HIT_NV,
        ));

        let descriptor_pool = self.generate_pool(&bindings)?;
        let descriptor_set_layout = self.generate_layout(&bindings)?;
        let descriptor_set = self.generate_set(descriptor_pool, descriptor_set_layout)?;

        Ok(DescriptorSet {
            device: Rc::clone(&self.context.get_device()),
            descriptor_pool,
            descriptor_set_layout,
            descriptor_set,
        })
    }

    fn cmd_pipeline_barrier(&self, command_buffer: vk::CommandBuffer, buffer: vk::Buffer) {
        let bmb = vk::BufferMemoryBarrier::builder()
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .offset(0)
            .size(vk::WHOLE_SIZE)
            .buffer(buffer)
            .build();

        self.context.get_device().cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::DependencyFlags::empty(),
            &[],
            &[bmb],
            &[],
        );
    }

    fn add_binding(
        &self,
        binding: u32,
        descriptor_count: u32,
        descriptor_type: vk::DescriptorType,
        stage: vk::ShaderStageFlags,
    ) -> vk::DescriptorSetLayoutBinding {
        vk::DescriptorSetLayoutBinding::builder()
            .binding(binding)
            .descriptor_count(descriptor_count)
            .descriptor_type(descriptor_type)
            .stage_flags(stage)
            .build()
    }

    fn generate_pool(
        &self,
        bindings: &[vk::DescriptorSetLayoutBinding],
    ) -> Result<vk::DescriptorPool, VulkanError> {
        let mut counters = vec![];
        for binding in bindings {
            counters.push(
                vk::DescriptorPoolSize::builder()
                    .ty(binding.descriptor_type)
                    .descriptor_count(binding.descriptor_count)
                    .build(),
            );
        }

        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&counters)
            .max_sets(1)
            .build();

        self.context.get_device().create_descriptor_pool(&pool_info)
    }

    fn generate_layout(
        &self,
        bindings: &[vk::DescriptorSetLayoutBinding],
    ) -> Result<vk::DescriptorSetLayout, VulkanError> {
        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(bindings)
            .build();
        self.context
            .get_device()
            .create_descriptor_set_layout(&layout_info)
    }

    fn generate_set(
        &self,
        pool: vk::DescriptorPool,
        layout: vk::DescriptorSetLayout,
    ) -> Result<vk::DescriptorSet, VulkanError> {
        let alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(pool)
            .set_layouts(&[layout])
            .build();

        self.context
            .get_device()
            .allocate_descriptor_sets(&alloc_info)
            .map(|set| set[0])
    }
}
