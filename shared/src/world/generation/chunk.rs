use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::time_noprint;
use crate::world::data::block::{BlockInstance, BlockManager, BlockType};
use crate::world::data::chunk::{CHUNK_BLOCK_NUMBER, CHUNK_SIZE, CHUNK_SIZE_F64, Chunk};
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
        // DEBUG PERLIN
        // let mut cave_perlin_times: Vec<f64> = Vec::with_capacity(CHUNK_BLOCK_NUMBER);

        // let mut empty_chunk: bool = true;

        let blocks = vec![BlockInstance::air(); CHUNK_BLOCK_NUMBER];

        let mut chunk = Chunk {
            blocks,
            x: cx,
            y: cy,
            z: cz,
        };

        let scale = 0.02;
        let base_height = 0;
        let amplitude = 10.0;

        let cx_f64 = cx as f64;
        let cz_f64 = cz as f64;

        for x in 0..CHUNK_SIZE {
            let x_f64 = x as f64;
            let wx = x_f64 + cx_f64 * CHUNK_SIZE_F64;
            let nx = wx * scale;
            for z in 0..CHUNK_SIZE {
                let z_f64 = z as f64;
                let wz = z_f64 + cz_f64 * CHUNK_SIZE_F64;
                let nz = wz * scale;

                let valeur = ctx.perlin.get([nx, nz]);
                let terrain_height = base_height as f64 + valeur * amplitude;
                let terrain_y = terrain_height as i32;

                for y in 0..CHUNK_SIZE {
                    let wy = y + cy * CHUNK_SIZE;
                    if wy >= terrain_y {
                        continue;
                    }

                    let depth = terrain_y - wy;
                    let wx_f = x_f64 + cx_f64 * CHUNK_SIZE_F64;
                    let wz_f = z_f64 + cz_f64 * CHUNK_SIZE_F64;
                    // let (is_cave, cave_time) = time_noprint!({
                    let is_cave=    ctx.is_cave_block(wx_f, wy as f64, wz_f, depth);
                    // });
                    // cave_perlin_times.push(cave_time.as_micros() as f64);

                    // "if is_cave" branch is not needed. Blocks are by default air.
                    if !is_cave {
                        // empty_chunk = false;
                        let block_id = match wy {
                            y if y == terrain_y - 1 => BlockType::Grass.to_u32(),
                            y if y >= terrain_y - 4 => BlockType::Dirt.to_u32(),
                            _ => BlockType::Stone.to_u32(),
                        };
                        chunk.set_block_from_xyz(x, y, z, BlockInstance::new(block_id));
                    }
                }
            }
        }

        // let (min, max, avg, sum) = if cave_perlin_times.is_empty() {
        //     (0.0, 0.0, 0.0, 0.0)
        // } else {
        //     let min = cave_perlin_times.iter().fold(f64::INFINITY, |arg0: f64, other: &f64| f64::min(arg0, *other));
        //     let max = cave_perlin_times.iter().fold(f64::NEG_INFINITY, |arg0: f64, other: &f64| f64::max(arg0, *other));
        //     let sum = cave_perlin_times.iter().sum::<f64>();
        //     let avg = sum / cave_perlin_times.len() as f64;
        //     (min, max, avg, sum)
        // };

        // println!("Chunk Generation (empty: {}) - Cave Perlin Noise:\nmin = {}µs\nmax = {}µs,\navg = {}µs,\nsum = {}ms", empty_chunk, min, max, avg, sum / 1000.0);

        chunk
    }
}
