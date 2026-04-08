use crate::game::world::data::block::BlockInstance;

pub const CHUNK_SIZE: i32 = 16;
pub const CHUNK_SIZE_F: f32 = CHUNK_SIZE as f32;
pub const CHUNK_SIZE_SQR: i32 = CHUNK_SIZE * CHUNK_SIZE;
pub const CHUNK_BLOCK_NUMBER: usize = (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize;
pub const LAST_CHUNK_AXIS_INDEX: i32 = CHUNK_SIZE - 1;
pub const LAST_CHUNK_AXIS_INDEX_USIZE: usize = LAST_CHUNK_AXIS_INDEX as usize;

#[derive(Clone, Copy, PartialEq)]
pub enum ChunkState {
    Pending,
    Ready,
}

pub struct ChunkData {
    pub chunk: Chunk,
    pub state: ChunkState,
    pub is_dirty: bool,
}

#[derive(Clone)]
pub struct Chunk {
    pub(crate) blocks: [BlockInstance; CHUNK_BLOCK_NUMBER],
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkData {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            state: ChunkState::Ready,
            is_dirty: true,
        }
    }

    pub fn set_dirty(&mut self) {
        if self.state == ChunkState::Ready {
            self.is_dirty = true;
        }
    }
}

impl Chunk {
    pub fn get_block_from_xyz(&self, x: i32, y: i32, z: i32) -> BlockInstance {
        return self.get_block_from_i((x + y * CHUNK_SIZE + z * CHUNK_SIZE_SQR) as usize);
    }

    pub fn get_block_from_i(&self, i: usize) -> BlockInstance {
        return self.blocks[i];
    }

    #[inline(always)]
    pub fn set_block_from_xyz(&mut self, x: i32, y: i32, z: i32, block: BlockInstance) {
        self.set_block_from_i((x + y * CHUNK_SIZE + z * CHUNK_SIZE_SQR) as usize, block);
    }

    #[inline(always)]
    pub fn set_block_from_i(&mut self, i: usize, block: BlockInstance) {
        self.blocks[i] = block;
    }
}
