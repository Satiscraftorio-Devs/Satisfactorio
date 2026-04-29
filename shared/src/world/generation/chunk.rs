use std::sync::Arc;

use crate::world::data::block::{BlockInstance, BlockManager, BlockType};
use crate::world::data::chunk::{Chunk, CHUNK_BLOCK_NUMBER, CHUNK_SIZE};
use crate::world::generation::chunk_generator::ChunkGenContext;
use noise::NoiseFn;

pub struct ChunkWithChecksum {
    pub chunk_data: crate::world::data::chunk::ChunkData,
    pub checksum: [u8; 2],
}

impl Chunk {
    #[inline]
    pub fn generate(block_manager: Arc<BlockManager>, cx: i32, cy: i32, cz: i32, seed: u32) -> Chunk {
        let ctx = ChunkGenContext::new(seed, block_manager);
        Self::generate_with_context(cx, cy, cz, &ctx)
    }

    #[inline]
    pub fn generate_with_context(cx: i32, cy: i32, cz: i32, ctx: &ChunkGenContext) -> Chunk {
        let mut blocks = Vec::with_capacity(CHUNK_BLOCK_NUMBER);
        for _ in 0..CHUNK_BLOCK_NUMBER {
            blocks.push(BlockInstance::air());
        }
        let mut chunk = Chunk {
            blocks,
            x: cx,
            y: cy,
            z: cz,
        };

        let scale = 0.02;
        let base_height = 16;
        let amplitude = 10.0;



        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let wx = x as f64 + cx as f64 * CHUNK_SIZE as f64;
                let wz = z as f64 + cz as f64 * CHUNK_SIZE as f64;
                let nx = wx * scale;
                let nz = wz * scale;

                let valeur = ctx.perlin.get([nx, nz]);
                let terrain_height = base_height as f64 + valeur * amplitude;
                let terrain_y = terrain_height as i32;

                for y in 0..CHUNK_SIZE {
                    let wy = y as i32 + cy as i32 * CHUNK_SIZE;
                    if wy < terrain_y {
                        let depth = terrain_y - wy;
                        let wx_f = x as f64 + cx as f64 * CHUNK_SIZE as f64;
                        let wz_f = z as f64 + cz as f64 * CHUNK_SIZE as f64;
                        let is_cave = ctx.is_cave_block(wx_f, wy as f64, wz_f, depth);

                        if is_cave {
                            chunk.set_block_from_xyz(x, y, z, BlockInstance::air());
                        } else {
                            let block_id = if wy == terrain_y - 1 {
                                BlockType::Grass as u32
                            } else if wy < terrain_y - 4 {
                                BlockType::Stone as u32
                            } else {
                                BlockType::Dirt as u32
                            };
                            chunk.set_block_from_xyz(x, y, z, BlockInstance::new(block_id));
                        }
                    }
                }
            }
        }

        chunk
    }
}
