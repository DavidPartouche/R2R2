use crate::input_manager::InputManager;
use std::cell::RefCell;
use std::rc::Rc;
use vulkan_ray_tracing::glm;
use winit::dpi::LogicalPosition;
use winit::event::VirtualKeyCode;
use winit::window::Window;

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
            position: glm::vec3(0.0, 0.0, 10.0),
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
    movement_speed: f32,
    rotation_speed: f32,
    yaw: f32,
    pitch: f32,
    mouse_grabbed: bool,
    last_mouse_position: LogicalPosition,
}

impl CameraManager {
    pub fn new(
        input_manager: Rc<RefCell<InputManager>>,
        width: f32,
        height: f32,
        camera_properties: CameraProperties,
    ) -> Self {
        let front = glm::vec3(0.0, 0.0, -1.0);
        let up = glm::vec3(0.0, 1.0, 0.0);
        let view = glm::look_at(
            &camera_properties.position,
            &(camera_properties.position + front),
            &up,
        );

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
            movement_speed: 2.0,
            rotation_speed: 50.0,
            yaw: -90.0,
            pitch: 0.0,
            mouse_grabbed: false,
            last_mouse_position: LogicalPosition::new(0.0, 0.0),
        }
    }

    pub fn get_camera_buffer(&self) -> &[u8] {
        let data = &self.camera as *const Camera as *const u8;
        unsafe { std::slice::from_raw_parts(data, std::mem::size_of::<Camera>()) }
    }

    pub fn get_camera_buffer_size(&self) -> usize {
        std::mem::size_of::<Camera>()
    }

    pub fn update(&mut self, window: &Window, mouse_position: &LogicalPosition, delta_time: f32) {
        // Hide the mouse when controlling the camera
        if !self.input_manager.borrow().is_right_button_down() {
            if self.mouse_grabbed {
                self.mouse_grabbed = false;
                window.set_cursor_grab(false).unwrap();
                window.set_cursor_visible(true);
                window
                    .set_cursor_position(self.last_mouse_position)
                    .unwrap();
            }
            return;
        }

        if !self.mouse_grabbed {
            self.mouse_grabbed = true;
            self.last_mouse_position = *mouse_position;
            window.set_cursor_grab(true).unwrap();
            window.set_cursor_visible(false);
        }

        // mouse movement
        let mouse_movement = self.input_manager.borrow().mouse_movement();
        self.yaw += mouse_movement.0 as f32 * delta_time * self.rotation_speed;
        self.pitch += mouse_movement.1 as f32 * delta_time * self.rotation_speed;

        self.pitch = self.pitch.min(89.0).max(-89.0);

        let front = glm::vec3(
            self.pitch.to_radians().cos() * self.yaw.to_radians().cos(),
            -self.pitch.to_radians().sin(),
            self.pitch.to_radians().cos() * self.yaw.to_radians().sin(),
        )
        .normalize();

        // keyboard press
        let up = glm::vec3(0.0, 1.0, 0.0);
        if self
            .input_manager
            .borrow()
            .is_key_pressed(VirtualKeyCode::S)
        {
            self.position -= front * delta_time * self.movement_speed;
        }
        if self
            .input_manager
            .borrow()
            .is_key_pressed(VirtualKeyCode::W)
        {
            self.position += front * delta_time * self.movement_speed;
        }
        if self
            .input_manager
            .borrow()
            .is_key_pressed(VirtualKeyCode::A)
        {
            self.position -= front.cross(&up).normalize() * delta_time * self.movement_speed;
        }
        if self
            .input_manager
            .borrow()
            .is_key_pressed(VirtualKeyCode::D)
        {
            self.position += front.cross(&up).normalize() * delta_time * self.movement_speed;
        }

        self.camera.view = glm::look_at(&self.position, &(self.position + front), &up);
        self.camera.view_inverse = glm::inverse(&self.camera.view);
    }
}
