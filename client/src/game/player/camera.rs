use std::f32::consts::FRAC_PI_2;

use winit::keyboard::KeyCode;

use crate::engine::render::camera::Camera;
use crate::game::player::player::Player;

pub struct CameraController {
    pub speed: f32,
    mouse_sensitivity: f32,
    pub is_forward_pressed: bool,
    pub is_backward_pressed: bool,
    pub is_left_pressed: bool,
    pub is_right_pressed: bool,
    pub is_up_pressed: bool,
    pub is_down_pressed: bool,
    mouse_delta_x: f32,
    mouse_delta_y: f32,
}

impl CameraController {
    pub fn new(speed: f32, mouse_sensitivity: f32) -> Self {
        Self {
            speed,
            mouse_sensitivity,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            mouse_delta_x: 0.0,
            mouse_delta_y: 0.0,
        }
    }

    pub fn handle_key(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        match code {
            KeyCode::KeyW | KeyCode::KeyZ => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyS => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::KeyQ => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyD => {
                self.is_right_pressed = is_pressed;
                true
            }
            KeyCode::Space => {
                self.is_up_pressed = is_pressed;
                true
            }
            KeyCode::ShiftLeft => {
                self.is_down_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, dx: f64, dy: f64) {
        self.mouse_delta_x += dx as f32;
        self.mouse_delta_y += dy as f32;
    }

    pub fn update_camera(&mut self, camera: &mut Camera, player: &Player) {
        camera.set_position(player.pos.current().clone());

        camera.yaw += self.mouse_delta_x * self.mouse_sensitivity;
        camera.pitch -= self.mouse_delta_y * self.mouse_sensitivity;
        self.mouse_delta_x = 0.0;
        self.mouse_delta_y = 0.0;

        camera.pitch = camera.pitch.clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);
    }
}
