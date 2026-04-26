use cgmath::Point3;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use shared::network::messages::{Position, Rotation};
use shared::parallel::{Parallelizable, WorkerPool};
use shared::world::data::chunk::Chunk;
use shared::world::generation::chunk::ChunkWithChecksum;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
struct ChunkGeneratorContext {
    seed: u32,
}

struct ChunkGenerator;

impl Parallelizable for ChunkGenerator {
    type Input = (i32, i32, i32);
    type Output = ChunkWithChecksum;
    type Context = ChunkGeneratorContext;

    fn process(input: Self::Input, ctx: &Self::Context) -> Self::Output {
        let (cx, cy, cz) = input;
        let chunk = Chunk::generate(cx, cy, cz, ctx.seed);
        let checksum = chunk.compute_checksum();
        let chunk_data = shared::world::data::chunk::ChunkData::new(chunk);
        ChunkWithChecksum { chunk_data, checksum }
    }
}

pub fn generate_chunks_parallel(seed: u32, coords: Vec<(i32, i32, i32)>) -> HashMap<(i32, i32, i32), ChunkWithChecksum> {
    let mut result_map = HashMap::new();
    if coords.is_empty() {
        return result_map;
    }

    let num_cpus = num_cpus::get();
    let pool = WorkerPool::<ChunkGenerator>::new(num_cpus, ChunkGeneratorContext { seed });

    for coord in &coords {
        let _ = pool.submit(*coord, *coord);
    }

    let mut received = 0;
    while received < coords.len() {
        if let Some(result) = pool.try_recv() {
            result_map.insert(result.coords, result.output);
            received += 1;
        }
    }

    drop(pool);

    result_map
}
