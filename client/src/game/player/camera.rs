use std::f32::consts::FRAC_PI_2;

use cgmath::{Deg, InnerSpace, Matrix4, Point3, Vector3};
use winit::keyboard::KeyCode;

use crate::{engine::render::camera::OPENGL_TO_WGPU_MATRIX, game::player::player::Player};

#[derive(Clone)]
pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub fovy: f32,
    pub aspect: f32,
    pub znear: f32,
    pub zfar: f32,
}

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

impl Camera {
    pub fn new(eye: cgmath::Point3<f32>, aspect: f32) -> Camera {
        Camera {
            eye,
            yaw: 0.0,
            pitch: 0.0,
            fovy: 70.0,
            aspect,
            znear: 0.1,
            zfar: 1000.0,
        }
    }

    pub fn forward(&self) -> Vector3<f32> {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();

        Vector3::new(cy * cp, sp, sy * cp).normalize()
    }

    pub fn right(&self) -> Vector3<f32> {
        self.forward().cross(Vector3::unit_y()).normalize()
    }

    pub fn target(&self) -> Point3<f32> {
        self.eye + self.forward()
    }

    pub fn get_view_proj(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(self.eye, self.target(), Vector3::unit_y());
        let proj = cgmath::perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);
        OPENGL_TO_WGPU_MATRIX * proj * view
    }

    pub fn set_position(&mut self, position: cgmath::Point3<f32>) {
        self.eye = position;
    }
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

    pub fn get_rotation(&self, camera: &Camera) -> (f32, f32) {
        (camera.yaw, camera.pitch)
    }
}
