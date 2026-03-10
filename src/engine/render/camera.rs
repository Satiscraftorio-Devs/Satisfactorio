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
    pub target: cgmath::Point3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
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
        // OPENGL_TO_WGPU_MATRIX * proj * view
        proj * view
    }

    pub fn set_position(&mut self, position: cgmath::Point3<f32>) {
        self.eye = position;
    }

    pub fn get_position(&self) -> cgmath::Point3<f32> {
        self.eye
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

    pub fn get_view_proj(&self) -> Matrix4<f32> {
        self.view_proj.into()
    }
}
