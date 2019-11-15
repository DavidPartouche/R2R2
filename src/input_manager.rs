use std::collections::HashSet;
use winit::event::{DeviceEvent, ElementState, VirtualKeyCode};

pub struct InputManager {
    key_inputs: HashSet<VirtualKeyCode>,
}

impl InputManager {
    pub fn new() -> Self {
        InputManager {
            key_inputs: HashSet::new(),
        }
    }

    pub fn update(&mut self, events: &[DeviceEvent]) {
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
                _ => {}
            }
        }
    }

    pub fn is_key_pressed(&self, keycode: VirtualKeyCode) -> bool {
        self.key_inputs.contains(&keycode)
    }
}
