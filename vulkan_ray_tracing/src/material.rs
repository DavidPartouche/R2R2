use nalgebra_glm as glm;

#[repr(C)]
pub struct Material {
    pub ambient: glm::Vec3,
    pub diffuse: glm::Vec3,
    pub specular: glm::Vec3,
    pub transmittance: glm::Vec3,
    pub emission: glm::Vec3,
    pub shininess: f32,
    pub ior: f32,
    pub dissolve: f32,
    pub illum: i32,
    pub texture_id: i32,
}

impl Default for Material {
    fn default() -> Self {
        Material {
            ambient: glm::vec3(0.1, 0.1, 0.1),
            diffuse: glm::vec3(0.7, 0.7, 0.7),
            specular: glm::vec3(1.0, 1.0, 1.0),
            transmittance: glm::vec3(0.0, 0.0, 0.0),
            emission: glm::vec3(0.0, 0.0, 0.1),
            shininess: 0.0,
            ior: 1.0,
            dissolve: 1.0,
            illum: 0,
            texture_id: -1,
        }
    }
}
