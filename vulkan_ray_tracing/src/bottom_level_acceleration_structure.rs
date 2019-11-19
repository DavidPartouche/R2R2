use ash::vk;

pub type BottomLevelAccelerationStructure = vk::GeometryNV;

pub struct BottomLevelAccelerationStructureBuilder {
    vertex_buffer: Option<vk::Buffer>,
    vertex_offset: vk::DeviceSize,
    vertex_count: u32,
    vertex_size: vk::DeviceSize,
    index_buffer: Option<vk::Buffer>,
    index_offset: vk::DeviceSize,
    index_count: u32,
    opaque: bool,
}

impl Default for BottomLevelAccelerationStructureBuilder {
    fn default() -> Self {
        BottomLevelAccelerationStructureBuilder {
            vertex_buffer: None,
            vertex_offset: 0,
            vertex_count: 0,
            vertex_size: 0,
            index_buffer: None,
            index_offset: 0,
            index_count: 0,
            opaque: false,
        }
    }
}

impl BottomLevelAccelerationStructureBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_vertex_buffer(mut self, buffer: vk::Buffer) -> Self {
        self.vertex_buffer = Some(buffer);
        self
    }

    pub fn with_vertex_offset(mut self, offset: u32) -> Self {
        self.vertex_offset = offset as vk::DeviceSize;
        self
    }

    pub fn with_vertex_count(mut self, count: u32) -> Self {
        self.vertex_count = count;
        self
    }

    pub fn with_vertex_size(mut self, size: u32) -> Self {
        self.vertex_size = size as vk::DeviceSize;
        self
    }

    pub fn with_index_buffer(mut self, buffer: vk::Buffer) -> Self {
        self.index_buffer = Some(buffer);
        self
    }

    pub fn with_index_offset(mut self, offset: u32) -> Self {
        self.index_offset = offset as vk::DeviceSize;
        self
    }

    pub fn with_index_count(mut self, count: u32) -> Self {
        self.index_count = count;
        self
    }

    pub fn with_opaque(mut self, opaque: bool) -> Self {
        self.opaque = opaque;
        self
    }

    pub fn build(self) -> BottomLevelAccelerationStructure {
        let triangles = vk::GeometryTrianglesNV::builder()
            .vertex_data(self.vertex_buffer.unwrap())
            .vertex_offset(self.vertex_offset)
            .vertex_count(self.vertex_count)
            .vertex_stride(self.vertex_size)
            .vertex_format(vk::Format::R32G32B32_SFLOAT)
            .index_data(self.index_buffer.unwrap())
            .index_offset(self.index_offset)
            .index_count(self.index_count)
            .index_type(vk::IndexType::UINT16)
            .transform_data(vk::Buffer::null())
            .transform_offset(0)
            .build();

        let flags = if self.opaque {
            vk::GeometryFlagsNV::OPAQUE
        } else {
            vk::GeometryFlagsNV::empty()
        };

        vk::GeometryNV::builder()
            .geometry_type(vk::GeometryTypeNV::TRIANGLES)
            .geometry(
                vk::GeometryDataNV::builder()
                    .triangles(triangles)
                    .aabbs(vk::GeometryAABBNV::default())
                    .build(),
            )
            .flags(flags)
            .build()
    }
}
