use vulkan_ray_tracing::glm;

type Transform = glm::Mat4;

#[repr(C, packed)]
struct Camera {
    view: Transform,
    proj: Transform,
    view_inverse: Transform,
    proj_inverse: Transform,
}

pub struct CameraManager {
    camera: Camera,
}

impl CameraManager {
    pub fn new(width: f32, height: f32) -> Self {
        let view = glm::look_at(
            &glm::vec3(4.0, 4.0, 4.0),
            &glm::vec3(0.0, 0.0, 0.0),
            &glm::vec3(0.0, 1.0, 0.0),
        );
        let aspect_ratio = width / height;
        let mut proj = glm::perspective(f32::to_radians(65.0), aspect_ratio, 0.1, 1000.0);
        proj[(1, 1)] = -proj[(1, 1)];
        let view_inverse = glm::inverse(&view);
        let proj_inverse = glm::inverse(&proj);

        Self {
            camera: Camera {
                view,
                proj,
                view_inverse,
                proj_inverse,
            },
        }
    }

    pub fn get_camera_buffer(&self) -> &[u8] {
        let data = &self.camera as *const Camera as *const u8;
        unsafe { std::slice::from_raw_parts(data, std::mem::size_of::<Camera>()) }
    }

    pub fn get_camera_buffer_size(&self) -> usize {
        std::mem::size_of::<Camera>()
    }

    pub fn update(&self, _delta_time: f32) {}
}
