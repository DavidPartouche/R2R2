use std::collections::HashSet;
use winit::event::{DeviceEvent, ElementState, VirtualKeyCode};

pub struct InputManager {
    key_inputs: HashSet<VirtualKeyCode>,
    mouse_delta: (f64, f64),
    left_button_down: bool,
    right_button_down: bool,
}

impl InputManager {
    pub fn new() -> Self {
        InputManager {
            key_inputs: HashSet::new(),
            mouse_delta: (0.0, 0.0),
            left_button_down: false,
            right_button_down: false,
        }
    }

    pub fn update(&mut self, events: &[DeviceEvent]) {
        self.mouse_delta = (0.0, 0.0);

        for event in events {
            match *event {
                DeviceEvent::Key(input) => {
                    if let Some(keycode) = input.virtual_keycode {
                        match input.state {
                            ElementState::Pressed => self.key_inputs.insert(keycode),
                            ElementState::Released => self.key_inputs.remove(&keycode),
                        };
                    }
                }
                DeviceEvent::MouseMotion { delta } => self.mouse_delta = delta,
                DeviceEvent::Button { button, state } => {
                    if button == 1 {
                        self.left_button_down = state == ElementState::Pressed;
                    } else if button == 3 {
                        self.right_button_down = state == ElementState::Pressed;
                    }
                }
                _ => {}
            }
        }
    }

    pub fn is_key_pressed(&self, keycode: VirtualKeyCode) -> bool {
        self.key_inputs.contains(&keycode)
    }

    pub fn mouse_movement(&self) -> (f64, f64) {
        self.mouse_delta
    }

    pub fn is_left_button_down(&self) -> bool {
        self.left_button_down
    }

    pub fn is_right_button_down(&self) -> bool {
        self.right_button_down
    }
}
