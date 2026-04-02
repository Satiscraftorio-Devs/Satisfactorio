use crate::game::world::chunk_generator::ChunkGenerator;
use cgmath::num_traits::ToPrimitive;
use noise::{Perlin, Seedable};
use rand::prelude::*;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{collections::HashMap, time::Instant};

use crate::{
    engine::render::{mesh::world::WorldMesh, render::RenderManager},
    game::{
        player::player::Player,
        world::{
            block::BlockInstance,
            chunk::{Chunk, ChunkData, CHUNK_SIZE},
        },
    },
};

pub struct World {
    chunks: HashMap<(i32, i32, i32), ChunkData>,
    pub perlin: Perlin,
    seed: u32,
    chunk_generator: ChunkGenerator,
}

impl World {
    pub fn new() -> World {
        let mut rng = rand::rng();
        let seed = rng.random::<u32>();
        let perlin = Perlin::default().set_seed(seed);
        let chunk_generator = ChunkGenerator::new(perlin.clone());

        return World {
            chunks: HashMap::new(),
            perlin: perlin,
            seed: seed,
            chunk_generator: chunk_generator,
        };
    }

    #[inline(always)]
    pub fn get_chunk_data(&self, cx: i32, cy: i32, cz: i32) -> Option<&ChunkData> {
        return self.chunks.get(&(cx, cy, cz));
    }

    #[inline(always)]
    pub fn get_chunk(&self, cx: i32, cy: i32, cz: i32) -> Option<&Chunk> {
        return self.chunks.get(&(cx, cy, cz)).map(|d| &d.chunk);
    }

    #[inline(always)]
    pub fn get_chunk_mut(&mut self, cx: i32, cy: i32, cz: i32) -> Option<&mut ChunkData> {
        return self.chunks.get_mut(&(cx, cy, cz));
    }

    pub fn set_chunk(&mut self, cx: i32, cy: i32, cz: i32, chunk: Chunk) {
        self.chunks.insert((cx, cy, cz), ChunkData::new(chunk));
    }

    pub fn update(&mut self, render_manager: &mut RenderManager, world_mesh: &mut WorldMesh, player: &Player) {
        if !player.pos.has_changed() {
            return;
        }

        let world_update_start = Instant::now();

        let needed_simulation_keys: Vec<(i32, i32, i32)> = player.get_simulation_chunk_keys();

        // println!("Time to get needed simulation keys: {:3}ms.", world_update_start.elapsed().as_millis());

        let world_update_start = Instant::now();

        let current_keys: Vec<_> = self.chunks.keys().cloned().collect();
        for key in current_keys {
            if !needed_simulation_keys.contains(&key) {
                self.chunks.remove(&key);
                if let Some(mesh) = world_mesh.meshes.remove(&key) {
                    if let Some(id) = mesh.mesh_id {
                        render_manager.release_mesh(id);
                    }
                }
            }
        }

        // println!("Time to unload chunks: {:3}ms.", world_update_start.elapsed().as_millis());

        // Identifier les chunks manquants
        let missing_keys: Vec<_> = needed_simulation_keys
            .iter()
            .filter(|k| !self.chunks.contains_key(k))
            .cloned()
            .collect();

        let player_cx = (player.get_pos().x / CHUNK_SIZE as f32).floor() as i32;
        let player_cz = (player.get_pos().z / CHUNK_SIZE as f32).floor() as i32;

        let mut sorted_missing: Vec<_> = missing_keys
            .into_iter()
            .map(|key| {
                let dx = key.0 - player_cx;
                let dz = key.2 - player_cz;
                let dist_2 = dx * dx + dz * dz;
                (key, dist_2)
            })
            .collect();

        // Trier par distance croissante (+proche IMPLIQUE +prioritaire)
        sorted_missing.sort_by(|a, b| a.1.cmp(&b.1));

        for (key, _) in sorted_missing {
            self.chunk_generator.request(key.0, key.1, key.2);
        }

        while let Ok(result) = self.chunk_generator.try_recv() {
            // Insérer le chunk généré dans le world
            self.chunks
                .insert((result.get_cx(), result.get_cy(), result.get_cz()), result.chunk_data);
        }

        // println!("Time to generate new chunks: {:3}ms.", world_update_start.elapsed().as_millis());
    }

    pub fn get_player_rendered_chunks(&self, player: &Player) -> Vec<&Chunk> {
        let [min_cx, max_cx, min_cy, max_cy, min_cz, max_cz] = player.get_rendered_chunk_range();

        let mut chunks: Vec<&Chunk> = Vec::new();

        for x in min_cx..=max_cx {
            for y in min_cy..=max_cy {
                for z in min_cz..=max_cz {
                    if let Some(data) = self.get_chunk_data(x, y, z) {
                        chunks.push(&data.chunk);
                    }
                }
            }
        }

        return chunks;
    }

    pub fn get_dirty_chunks(&self) -> Vec<(i32, i32, i32)> {
        self.chunks.iter().filter(|(_, data)| data.is_dirty).map(|(key, _)| *key).collect()
    }

    pub fn mark_chunk_clean(&mut self, cx: i32, cy: i32, cz: i32) {
        if let Some(data) = self.chunks.get_mut(&(cx, cy, cz)) {
            data.is_dirty = false;
        }
    }

    pub fn get_block_from_xyz(&self, x: i32, y: i32, z: i32) -> BlockInstance {
        let cx: i32 = x.div_euclid(CHUNK_SIZE);
        let cy: i32 = y.div_euclid(CHUNK_SIZE);
        let cz: i32 = z.div_euclid(CHUNK_SIZE);

        let cbx: i32 = x.rem_euclid(CHUNK_SIZE);
        let cby: i32 = y.rem_euclid(CHUNK_SIZE);
        let cbz: i32 = z.rem_euclid(CHUNK_SIZE);

        if let Some(data) = self.get_chunk_data(cx, cy, cz) {
            return data.chunk.get_block_from_xyz(cbx, cby, cbz);
        } else {
            return BlockInstance::air();
        }
    }

    pub fn get_local_block_from_xyz(&self, lx: i32, ly: i32, lz: i32, cx: i32, cy: i32, cz: i32) -> BlockInstance {
        if !(0..CHUNK_SIZE).contains(&lx) || !(0..CHUNK_SIZE).contains(&ly) || !(0..CHUNK_SIZE).contains(&lz) {
            return self.get_block_from_xyz(lx + cx * CHUNK_SIZE, ly + cy * CHUNK_SIZE, lz + cz * CHUNK_SIZE);
        }

        if let Some(data) = self.get_chunk_data(cx, cy, cz) {
            return data.chunk.get_block_from_xyz(lx, ly, lz);
        } else {
            return BlockInstance::air();
        }
    }
}
