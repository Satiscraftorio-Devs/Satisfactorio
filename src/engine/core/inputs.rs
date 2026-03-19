use winit::keyboard::{KeyCode, PhysicalKey};

pub struct InputState {
    mouse_delta: (f64, f64),
    pressed_keys: Vec<KeyCode>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            mouse_delta: (0.0, 0.0),
            pressed_keys: vec![],
        }
    }

    pub fn set_key_press(&mut self, key: KeyCode) {
        self.pressed_keys.push(key);
    }

    pub fn is_key_pressed(&mut self, key: KeyCode) -> bool {
        return self.pressed_keys.contains(&key);
    }

    pub fn clear_keys(&mut self) {
        self.pressed_keys.clear();
    }

    pub fn set_mouse_delta(&mut self, delta: (f64, f64)) {
        self.mouse_delta = delta;
    }

    pub fn get_mouse_delta(&self) -> (f64, f64) {
        return self.mouse_delta;
    }
}