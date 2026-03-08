use crate::player::Player;
use cgmath::{Deg, InnerSpace, Matrix4, Point3, Vector3};
use winit::keyboard::KeyCode;

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

#[derive(Clone)]
pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    pub fn new(
        eye: cgmath::Point3<f32>,
        target: cgmath::Point3<f32>,
        up: cgmath::Vector3<f32>,
        aspect: f32,

        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Camera {
        Camera {
            eye: eye,
            target: target,
            up: up,
            aspect: aspect,
            yaw: 0.0,
            pitch: 0.0,
            fovy: fovy,
            znear: znear,
            zfar: zfar,
        }
    }

    pub fn forward(&self) -> Vector3<f32> {
        // yaw: rotation autour de Y, pitch: rotation autour de X
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();

        Vector3::new(
            cy * cp, // x
            sp,      // y
            sy * cp, // z
        )
        .normalize()
    }

    pub fn right(&self) -> Vector3<f32> {
        self.forward().cross(Vector3::unit_y()).normalize()
    }

    pub fn target(&self) -> Point3<f32> {
        self.eye + self.forward()
    }

    fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(self.eye, self.target(), Vector3::unit_y());
        let proj = cgmath::perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);
        OPENGL_TO_WGPU_MATRIX * proj * view
    }

    pub fn set_position(&mut self, position: cgmath::Point3<f32>) {
        self.eye = position;
    }

    pub fn get_position(&self) -> cgmath::Point3<f32> {
        self.eye
    }
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
    pub mouse_delta_x: f32,
    pub mouse_delta_y: f32,
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
            // ZQSD + WASD
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

            // Vertical (optionnel)
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

    pub fn update_camera(&mut self, dt: f32, camera: &mut Camera, player: &Player) {
        // 1. Sync position joueur → camera (TOUJOURS)
        camera.eye = player.pos;

        // 2. Rotation souris (yaw/pitch) UNIQUEMENT
        camera.yaw += self.mouse_delta_x * self.mouse_sensitivity * dt;
        camera.pitch -= self.mouse_delta_y * self.mouse_sensitivity * dt;
        self.mouse_delta_x = 0.0;
        self.mouse_delta_y = 0.0;

        // Clamp pitch
        let max_pitch = std::f32::consts::FRAC_PI_2 - 0.01;
        camera.pitch = camera.pitch.clamp(-max_pitch, max_pitch);
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
