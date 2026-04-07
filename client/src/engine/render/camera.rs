use cgmath::{Deg, InnerSpace, Matrix4, Point3, Vector3};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

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
        proj * view
    }

    pub fn set_position(&mut self, position: cgmath::Point3<f32>) {
        self.eye = position;
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

    pub fn get_view_proj_raw(&self) -> [[f32; 4]; 4] {
        self.view_proj
    }

    pub fn get_view_proj(&self) -> Matrix4<f32> {
        self.view_proj.into()
    }
}
