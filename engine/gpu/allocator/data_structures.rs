use crate::gpu::allocator::gpu_allocator::MeshId;
use std::fmt::Display;

#[repr(u8)]
#[derive(Debug)]
pub enum AllocError {
    InvalidId = 0,
    NotEnoughSpace = 1,
}

#[derive(Clone)]
pub struct Gap {
    pub position: usize,
    pub length: usize,
}

pub struct WriteOperation {
    pub mesh_id: MeshId,
    pub offset: usize,
    pub len: usize,
    pub arena_offset: usize,
}

impl Display for AllocError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            Self::InvalidId => "InvalidId",
            Self::NotEnoughSpace => "NotEnoughSpace",
        })
    }
}

impl Gap {
    pub fn new(position: usize, length: usize) -> Self {
        Self { position, length }
    }
}

impl WriteOperation {
    pub fn new(mesh_id: MeshId, offset: usize, len: usize, arena_offset: usize) -> Self {
        Self {
            mesh_id,
            offset,
            len,
            arena_offset,
        }
    }
}
