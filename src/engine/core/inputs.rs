use std::collections::HashMap;

use winit::keyboard::KeyCode;

pub struct InputState {
    mouse_delta: (f64, f64),
    pressed_keys: HashMap<KeyCode, bool>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            mouse_delta: (0.0, 0.0),
            pressed_keys: HashMap::new(),
        }
    }

    pub fn set_key_press(&mut self, key: KeyCode, is_pressed: bool) {
        self.pressed_keys.insert(key, is_pressed);
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        return *self.pressed_keys.get(&key).unwrap_or(&false);
    }

    pub fn set_mouse_delta(&mut self, delta: (f64, f64)) {
        self.mouse_delta = delta;
    }

    pub fn get_mouse_delta(&self) -> (f64, f64) {
        return self.mouse_delta;
    }
}
