use std::mem::size_of;

use ash::vk;

#[repr(C, packed)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub norm: [f32; 3],
    pub color: [f32; 3],
    pub tex_coord: [f32; 2],
}

impl Vertex {
    pub(crate) fn get_binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    pub(crate) fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 4] {
        let pos_attribute_description = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(memoffset::offset_of!(Vertex, pos) as u32)
            .build();

        let norm_attribute_description = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(memoffset::offset_of!(Vertex, norm) as u32)
            .build();

        let color_attribute_description = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(2)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(memoffset::offset_of!(Vertex, color) as u32)
            .build();

        let text_coord_attribute_description = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(3)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(memoffset::offset_of!(Vertex, tex_coord) as u32)
            .build();

        [
            pos_attribute_description,
            norm_attribute_description,
            color_attribute_description,
            text_coord_attribute_description,
        ]
    }
}
