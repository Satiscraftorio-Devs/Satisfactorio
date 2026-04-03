use noise::{NoiseFn, Perlin};

use crate::game::world::block::BlockInstance;

pub const CHUNK_SIZE: i32 = 32;
pub const CHUNK_USIZE: usize = CHUNK_SIZE as usize;
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

#[derive(Clone)]
pub struct Chunk {
    blocks: [BlockInstance; CHUNK_BLOCK_NUMBER],
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Chunk {
    pub fn generate(cx: i32, cy: i32, cz: i32, perlin: &Perlin) -> Chunk {
        let mut chunk = Chunk {
            blocks: [BlockInstance::air(); CHUNK_BLOCK_NUMBER],
            x: cx,
            y: cy,
            z: cz,
        };
        let block = BlockInstance::new(1);

        let scale = 0.01;

        let base_height = 16;
        let amplitude = 10.0;

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let wx = x as f64 + cx as f64 * CHUNK_SIZE as f64;
                let wz = z as f64 + cz as f64 * CHUNK_SIZE as f64;

                let nx = wx * scale;
                let nz = wz * scale;

                let valeur = perlin.get([nx, nz]);

                let terrain_height = base_height as f64 + valeur * amplitude;
                let terrain_y = terrain_height as i32;

                for y in 0..CHUNK_SIZE {
                    let wy = y as i32 + cy as i32 * CHUNK_SIZE;
                    if wy < terrain_y {
                        chunk.set_block_from_xyz(x, y, z, block.clone());
                    }
                }
            }
        }

        return chunk;
    }

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
