use crate::input_manager::InputManager;
use std::cell::RefCell;
use std::rc::Rc;
use vulkan_ray_tracing::glm;
use winit::event::VirtualKeyCode;

type Transform = glm::Mat4;

#[repr(C)]
struct Camera {
    view: Transform,
    proj: Transform,
    view_inverse: Transform,
    proj_inverse: Transform,
}

pub enum CameraType {
    Orthographic,
    Perspective,
}

pub struct CameraProperties {
    pub position: glm::Vec3,
    pub camera_type: CameraType,
    pub near: f32,
    pub far: f32,
}

impl Default for CameraProperties {
    fn default() -> Self {
        CameraProperties {
            position: glm::vec3(0.0, 0.0, -10.0),
            camera_type: CameraType::Perspective,
            near: 0.1,
            far: 1000.0,
        }
    }
}

pub struct CameraManager {
    input_manager: Rc<RefCell<InputManager>>,
    camera: Camera,
    position: glm::Vec3,
}

impl CameraManager {
    pub fn new(
        input_manager: Rc<RefCell<InputManager>>,
        width: f32,
        height: f32,
        camera_properties: CameraProperties,
    ) -> Self {
        let view = glm::translate(&glm::identity(), &camera_properties.position);

        let aspect_ratio = width / height;
        let mut proj = match camera_properties.camera_type {
            CameraType::Perspective => glm::perspective(
                f32::to_radians(65.0),
                aspect_ratio,
                camera_properties.near,
                camera_properties.far,
            ),
            CameraType::Orthographic => glm::ortho(
                0.0,
                width,
                0.0,
                height,
                camera_properties.near,
                camera_properties.far,
            ),
        };

        proj[(1, 1)] = -proj[(1, 1)];
        let view_inverse = glm::inverse(&view);
        let proj_inverse = glm::inverse(&proj);

        Self {
            input_manager,
            camera: Camera {
                view,
                proj,
                view_inverse,
                proj_inverse,
            },
            position: camera_properties.position,
        }
    }

    pub fn get_camera_buffer(&self) -> &[u8] {
        let data = &self.camera as *const Camera as *const u8;
        unsafe { std::slice::from_raw_parts(data, std::mem::size_of::<Camera>()) }
    }

    pub fn get_camera_buffer_size(&self) -> usize {
        std::mem::size_of::<Camera>()
    }

    pub fn update(&mut self, delta_time: f32) {
        if self
            .input_manager
            .borrow()
            .is_key_pressed(VirtualKeyCode::S)
        {
            self.position.z -= 1.0 * delta_time;
        }
        if self
            .input_manager
            .borrow()
            .is_key_pressed(VirtualKeyCode::W)
        {
            self.position.z += 1.0 * delta_time;
        }
        if self
            .input_manager
            .borrow()
            .is_key_pressed(VirtualKeyCode::A)
        {
            self.position.x += 1.0 * delta_time;
        }
        if self
            .input_manager
            .borrow()
            .is_key_pressed(VirtualKeyCode::D)
        {
            self.position.x -= 1.0 * delta_time;
        }
        if self
            .input_manager
            .borrow()
            .is_key_pressed(VirtualKeyCode::Q)
        {
            self.position.y -= 1.0 * delta_time;
        }
        if self
            .input_manager
            .borrow()
            .is_key_pressed(VirtualKeyCode::E)
        {
            self.position.y += 1.0 * delta_time;
        }

        self.camera.view = glm::translate(&glm::identity(), &self.position);
        self.camera.view_inverse = glm::inverse(&self.camera.view);
    }
}
