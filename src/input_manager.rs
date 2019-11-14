use winit::event::DeviceEvent;

pub struct InputManager {
    mouse_delta: (f64, f64),
}

impl InputManager {
    pub fn new() -> Self {
        InputManager {
            mouse_delta: (0.0, 0.0),
        }
    }

    pub fn update(&mut self, events: &[DeviceEvent]) {
        for event in events {
            match *event {
                DeviceEvent::MouseMotion { delta } => {
                    self.mouse_delta = delta;
                }
                DeviceEvent::Key(_) => {}
                _ => {}
            }
        }
    }
}
