#[repr(C)]
pub struct Material {
    pub ambient: [f32; 3],
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub transmittance: [f32; 3],
    pub emission: [f32; 3],
    pub shininess: f32,
    pub ior: f32,
    pub dissolve: f32,
    pub illum: i32,
    pub texture_id: i32,
}

impl Default for Material {
    fn default() -> Self {
        Material {
            ambient: [0.1, 0.1, 0.1],
            diffuse: [0.7, 0.7, 0.7],
            specular: [1.0, 1.0, 1.0],
            transmittance: [0.0, 0.0, 0.0],
            emission: [0.0, 0.0, 0.1],
            shininess: 0.0,
            ior: 1.0,
            dissolve: 1.0,
            illum: 0,
            texture_id: -1,
        }
    }
}
