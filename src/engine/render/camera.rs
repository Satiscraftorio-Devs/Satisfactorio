use cgmath::{Deg, InnerSpace, Matrix4, Point3, Vector3};

use crate::engine::render::render::RenderOptions;

// #[rustfmt::skip]
// pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
//     cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
//     cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
//     cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
//     cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
// );

#[derive(Clone)]
pub struct Camera {
    pub position: cgmath::Point3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub fovy: f32,
}

impl Camera {
    pub fn new(position: cgmath::Point3<f32>, fovy: f32) -> Camera {
        Camera {
            position,
            fovy: fovy,
            yaw: 0.0,
            pitch: 0.0,
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

    pub fn up(&self) -> Vector3<f32> {
        self.right().cross(self.forward()).normalize()
    }

    pub fn target(&self) -> Point3<f32> {
        self.position + self.forward()
    }

    pub fn get_view_proj(&mut self, render_options: &RenderOptions) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(self.position, self.target(), Vector3::unit_y());
        let proj = cgmath::perspective(Deg(self.fovy), render_options.aspect, render_options.znear, render_options.zfar);
        proj * view
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderCamera {
    view_proj: [[f32; 4]; 4],
}

impl RenderCamera {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, view_proj: Matrix4<f32>) {
        self.view_proj = view_proj.into();
    }

    pub fn get_view_proj(&self) -> Matrix4<f32> {
        self.view_proj.into()
    }

    pub fn get_view_proj_raw(&self) -> [[f32; 4]; 4] {
        self.view_proj
    }
}
