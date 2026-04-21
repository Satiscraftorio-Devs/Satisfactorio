use crate::parallel::{Parallelizable, QueueFull, WorkResult, WorkerPool};
use crate::world::data::block::{BlockInstance, BlockType};
use crate::world::data::chunk::{Chunk, ChunkData, CHUNK_BLOCK_NUMBER, CHUNK_SIZE};
use noise::{NoiseFn, Perlin, Seedable};

const CAVE_SCALE: f64 = 0.025;
const CAVE_THRESHOLD: f64 = 0.04;
const CAVE_MIN_DEPTH: i32 = 0;

pub struct ChunkWithChecksum {
    pub chunk_data: ChunkData,
    pub checksum: [u8; 2],
}

pub struct ChunkGen;

impl Parallelizable for ChunkGen {
    type Input = (i32, i32, i32);
    type Output = (i32, i32, i32, ChunkWithChecksum);
    type Context = u32;

    fn process(input: Self::Input, ctx: &Self::Context) -> Self::Output {
        let (cx, cy, cz) = input;
        let chunk = Chunk::generate(cx, cy, cz, *ctx);
        let checksum = chunk.compute_checksum();
        let chunk_data = ChunkData::new(chunk);
        (cx, cy, cz, ChunkWithChecksum { chunk_data, checksum })
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

    pub fn with_max_pending(seed: u32, max_pending: usize) -> Self {
        Self {
            inner: WorkerPool::with_max_pending(num_cpus::get(), seed, Some(max_pending)),
        }
    }

    pub fn request(&self, cx: i32, cy: i32, cz: i32) -> Result<(), QueueFull> {
        self.inner.submit((cx, cy, cz), (cx, cy, cz))
    }

    pub fn try_recv(&self) -> Option<WorkResult<(i32, i32, i32, ChunkWithChecksum)>> {
        self.inner.try_recv()
    }
}

fn is_cave_block(wx: f64, wy: f64, wz: f64, cave_noise_1: &Perlin, cave_noise_2: &Perlin, depth: i32) -> bool {
    if depth < CAVE_MIN_DEPTH {
        return false;
    }
    let nx = wx * CAVE_SCALE;
    let ny = wy * CAVE_SCALE;
    let nz = wz * CAVE_SCALE;
    let cave1 = cave_noise_1.get([nx, ny, nz]).abs();
    let cave2 = cave_noise_2.get([nx, ny, nz]).abs();
    cave1 < CAVE_THRESHOLD && cave2 < CAVE_THRESHOLD
}

impl Chunk {
    pub fn generate(cx: i32, cy: i32, cz: i32, seed: u32) -> Chunk {
        let perlin = Perlin::default().set_seed(seed);
        let cave_noise_1 = Perlin::default().set_seed(seed.wrapping_add(1000));
        let cave_noise_2 = Perlin::default().set_seed(seed.wrapping_add(2000));

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

                let valeur = perlin.get([nx, nz]);
                let terrain_height = base_height as f64 + valeur * amplitude;
                let terrain_y = terrain_height as i32;

                for y in 0..CHUNK_SIZE {
                    let wy = y as i32 + cy as i32 * CHUNK_SIZE;
                    if wy < terrain_y {
                        let depth = terrain_y - wy;
                        let wx_f = x as f64 + cx as f64 * CHUNK_SIZE as f64;
                        let wz_f = z as f64 + cz as f64 * CHUNK_SIZE as f64;
                        let is_cave = is_cave_block(wx_f, wy as f64, wz_f, &cave_noise_1, &cave_noise_2, depth);

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
