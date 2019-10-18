use ash::vk;

use crate::buffer::Buffer;

pub struct AccelerationStructure {
    scratch_buffer: Buffer,
    result_buffer: Buffer,
    instances_buffer: Buffer,
    structure: vk::AccelerationStructureNV,
}
