use cgmath::Matrix4;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

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
