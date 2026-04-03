use crate::common::utils::parallel::{Parallelizable, WorkResult, WorkerPool};
use crate::game::world::chunk::Chunk;
use crate::game::world::chunk::ChunkData;
use noise::Perlin;

pub struct ChunkGen;

impl Parallelizable for ChunkGen {
    type Input = (i32, i32, i32);
    type Output = (i32, i32, i32, ChunkData);
    type Context = Perlin;

    fn process(input: Self::Input, ctx: &Self::Context) -> Self::Output {
        let (cx, cy, cz) = input;
        let chunk = Chunk::generate(cx, cy, cz, ctx);
        (cx, cy, cz, ChunkData::new(chunk))
    }
}

pub struct ChunkGenerator {
    inner: WorkerPool<ChunkGen>,
}

impl ChunkGenerator {
    pub fn new(perlin: Perlin) -> Self {
        Self {
            inner: WorkerPool::new(num_cpus::get(), perlin),
        }
    }

    pub fn request(&self, cx: i32, cy: i32, cz: i32) {
        self.inner.submit((cx, cy, cz), (cx, cy, cz));
    }

    pub fn try_recv(&self) -> Option<WorkResult<(i32, i32, i32, ChunkData)>> {
        self.inner.try_recv()
    }
}
