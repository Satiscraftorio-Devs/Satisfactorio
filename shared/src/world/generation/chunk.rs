use crate::parallel::{Parallelizable, WorkResult, WorkerPool};
use crate::world::data::block::{BlockInstance, BlockType};
use crate::world::data::chunk::{Chunk, ChunkData, CHUNK_BLOCK_NUMBER, CHUNK_SIZE};
use noise::{NoiseFn, Perlin, Seedable};

pub struct ChunkGen;

impl Parallelizable for ChunkGen {
    type Input = (i32, i32, i32);
    type Output = (i32, i32, i32, ChunkData);
    type Context = u32;

    fn process(input: Self::Input, ctx: &Self::Context) -> Self::Output {
        let (cx, cy, cz) = input;
        let chunk = Chunk::generate(cx, cy, cz, *ctx);
        (cx, cy, cz, ChunkData::new(chunk))
    }
}

pub struct ChunkGenerator {
    inner: WorkerPool<ChunkGen>,
}

impl ChunkGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            inner: WorkerPool::new(num_cpus::get(), seed),
        }
    }

    pub fn request(&self, cx: i32, cy: i32, cz: i32) {
        self.inner.submit((cx, cy, cz), (cx, cy, cz));
    }

    pub fn try_recv(&self) -> Option<WorkResult<(i32, i32, i32, ChunkData)>> {
        self.inner.try_recv()
    }
}

impl Chunk {
    pub fn generate(cx: i32, cy: i32, cz: i32, seed: u32) -> Chunk {
        let perlin = Perlin::default().set_seed(seed);
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

        return chunk;
    }
}
