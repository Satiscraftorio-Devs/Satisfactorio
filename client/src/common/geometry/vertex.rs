use bytemuck::{Pod, Zeroable};

/// The color to display the world in, in RGB format.
///
/// Components must be between 0 and 1 inclusive.
pub const BASE_VERTEX_COLOR: [f32; 3] = [0.1, 0.7, 0.9];

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
    tex_layer: f32,
    ao: f32,
    u: f32,
    v: f32,
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32, tex_layer: u32, ao: f32, u: f32, v: f32) -> Vertex {
        return Vertex {
            position: [x, y, z],
            color: BASE_VERTEX_COLOR,
            tex_layer: tex_layer as f32,
            ao: ao,
            u,
            v,
        };
    }

    pub fn new_with_rgb(x: f32, y: f32, z: f32, r: f32, g: f32, b: f32, tex_layer: u32, ao: f32, u: f32, v: f32) -> Vertex {
        return Vertex {
            position: [x, y, z],
            color: [r, g, b],
            tex_layer: tex_layer as f32,
            ao: ao,
            u,
            v,
        };
    }

    pub fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress * 2,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress * 2 + mem::size_of::<f32>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress * 2 + mem::size_of::<f32>() as wgpu::BufferAddress * 2,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
