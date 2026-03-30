use std::f32::consts::FRAC_PI_2;

use cgmath::Vector3;
use winit::keyboard::KeyCode;

use crate::engine::{core::inputs::InputState, render::camera::Camera};

pub struct CameraController {
    pub speed: f32,
    pub mouse_sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, mouse_sensitivity: f32) -> Self {
        Self { speed, mouse_sensitivity }
    }

    pub fn get_delta_position(&self, camera: &Camera, inputs: &InputState) -> Vector3<f32> {
        let forward = camera.forward();
        let right = camera.right();
        let up = camera.up();

        let mut delta_position = Vector3::new(0.0, 0.0, 0.0);

        // ZQSD + WASD
        if inputs.is_key_pressed(KeyCode::KeyW) || inputs.is_key_pressed(KeyCode::KeyZ) {
            delta_position += forward;
        }
        if inputs.is_key_pressed(KeyCode::KeyS) {
            delta_position -= forward;
        }
        if inputs.is_key_pressed(KeyCode::KeyA) || inputs.is_key_pressed(KeyCode::KeyQ) {
            delta_position -= right;
        }
        if inputs.is_key_pressed(KeyCode::KeyD) {
            delta_position += right;
        }

        // Vertical (optionnel)
        if inputs.is_key_pressed(KeyCode::Space) {
            delta_position += up;
        }
        if inputs.is_key_pressed(KeyCode::ShiftLeft) {
            delta_position -= up;
        }

        delta_position * self.speed
    }

    pub fn get_delta_mouse_position(&self, inputs: &InputState) -> (f32, f32) {
        let (dx, dy) = inputs.get_mouse_delta();

        let yaw = (dx as f32) * self.mouse_sensitivity;
        let pitch = -(dy as f32) * self.mouse_sensitivity;

        return (yaw, pitch);
    }

    pub fn update_camera(&mut self, dt: f32, camera: &mut Camera, inputs: &InputState) {
        camera.position += self.get_delta_position(camera, inputs) * dt;

        const CLAMPED_PITCH: f32 = FRAC_PI_2 - 0.01;
        let (yaw, pitch) = self.get_delta_mouse_position(inputs);

        camera.yaw += yaw;
        camera.pitch += pitch;
        camera.pitch = camera.pitch.clamp(-CLAMPED_PITCH, CLAMPED_PITCH);
    }
}
