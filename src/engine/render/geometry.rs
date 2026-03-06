use bytemuck::{Pod, Zeroable};

/// The color to display the world in, in RGB format.
/// 
/// Components must be between 0 and 1 inclusive.
pub const BASE_VERTEX_COLOR: [f32; 3] = [0.5, 0.5, 0.5]; // Gris neutre

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
    uv: u32,
    // tex_coords: [f32; 2],
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32, uv: u32) -> Vertex {
        return Vertex {
            position: [x, y, z],
            color: BASE_VERTEX_COLOR,
            uv: uv
        }
    }

    pub fn new_with_rgb(x: f32, y: f32, z: f32, r: f32, g: f32, b: f32, uv: u32) -> Vertex {
        return Vertex {
            position: [x, y, z],
            color: [r, g, b],
            uv: uv
        }
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
            ],
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Face {
    Above = 0,
    Below = 1,
    Left  = 2,
    Right = 3,
    Front = 4,
    Back  = 5,
}

#[derive(Clone, Copy)]
pub struct FaceMask {
    pub data: u64
}

const VISITED_SHIFT: u64 = 63;
const BLOCK_ID_SHIFT: u64 = 31;
const BLOCK_ID_MASK: u64 = 0xFFFF_FFFF;
const FACE_MASK: u64 = 0b111;

impl Face {
    #[inline(always)]
    pub fn from_bits_unchecked(v: u8) -> Self {
        debug_assert!(v < 6);
        unsafe { std::mem::transmute(v) }
    }

    #[inline(always)]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }
}

impl FaceMask {
    #[inline(always)]
    pub fn empty() -> FaceMask {
        return FaceMask {
            data: 0x8000_0000_0000_0000u64,
        };
    }

    pub fn from(visited: bool, id: u32, face: Face) -> FaceMask {
        let mut mask = FaceMask::empty();
        mask.set_visited(visited);
        mask.set_block_id(id);
        mask.set_face(face);
        return mask;
    }

    pub fn to(&self) -> (bool, u32, Face) {
        return (self.get_visited(), self.get_block_id(), self.get_face());
    }

    #[inline(always)]
    pub fn get_visited(self) -> bool {
        (self.data >> VISITED_SHIFT) != 0
    }

    #[inline(always)]
    pub fn set_visited(&mut self, v: bool) {
        self.data ^= (-(v as i64) as u64 ^ self.data)
            & (1 << VISITED_SHIFT);
    }

    #[inline(always)]
    pub fn get_block_id(self) -> u32 {
        ((self.data >> BLOCK_ID_SHIFT) & BLOCK_ID_MASK) as u32
    }

    #[inline(always)]
    pub fn set_block_id(&mut self, id: u32) {
        self.data =
            (self.data & !(BLOCK_ID_MASK << BLOCK_ID_SHIFT))
            | ((id as u64) << BLOCK_ID_SHIFT);
    }

    #[inline(always)]
    pub fn get_face(self) -> Face {
        Face::from_bits_unchecked((self.data & FACE_MASK) as u8)
    }

    #[inline(always)]
    pub fn set_face(&mut self, face: Face) {
        self.data =
            (self.data & !FACE_MASK)
            | (face as u64);
    }
}