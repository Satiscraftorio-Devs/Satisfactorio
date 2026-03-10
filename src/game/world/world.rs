use std::collections::HashMap;

use crate::{engine::render::mesh::world::WorldMesh, game::{player::player::Player, world::{block::BlockInstance, chunk::{CHUNK_SIZE, Chunk}}}};

pub struct World {
    chunks: HashMap<(i32, i32, i32), Chunk>,
}

impl World {
    pub fn new() -> World {
        return World {
            chunks: HashMap::new()
        };
    }

    #[inline(always)]
    pub fn get_chunk(&self, cx: i32, cy: i32, cz: i32) -> Option<&Chunk> {
        return self.chunks.get(&(cx, cy, cz));
    }

    #[inline(always)]
    pub fn set_chunk(&mut self, cx: i32, cy: i32, cz: i32, chunk: Chunk) {
        self.chunks.insert((cx, cy, cz), chunk);
    }

    pub fn update(&mut self, player: &Player, world_mesh: &mut WorldMesh) {
        let ([min_cx, max_cx, min_cy, max_cy, min_cz, max_cz], chunk_number) = player.get_rendered_chunk_data();

        let mut chunks: Vec<&Chunk> = Vec::new();
        chunks.reserve_exact(chunk_number as usize);

        for x in min_cx..=max_cx {
            for y in min_cy..=max_cy {
                for z in min_cz..=max_cz {
                    if let Some(_chunk) = self.get_chunk(x, y, z) {
                        if let Some(chunk_mesh) = world_mesh.meshes.get(&(x, y, z)) {
                            chunk_mesh.set_dirty();
                        }
                    }
                    else {
                        let chunk = Chunk::generate(x, y, z);
                        self.set_chunk(x, y, z, chunk);
                    }
                }
            }
        }
    }

    pub fn get_player_rendered_chunks(&self, player: &Player) -> Vec<&Chunk> {
        let ([min_cx, max_cx, min_cy, max_cy, min_cz, max_cz], chunk_number) = player.get_rendered_chunk_data();

        let mut chunks: Vec<&Chunk> = Vec::new();
        chunks.reserve_exact(chunk_number as usize);

        for x in min_cx..=max_cx {
            for y in min_cy..=max_cy {
                for z in min_cz..=max_cz {
                    if let Some(chunk) = self.get_chunk(x, y, z) {
                        chunks.push(chunk);
                    }
                }
            }
        }

        return chunks;
    }

    pub fn get_block_from_xyz(&self, x: i32, y: i32, z: i32) -> BlockInstance {
        // Chunk coordinates
        let cx: i32 = x.div_euclid(CHUNK_SIZE);
        let cy: i32 = y.div_euclid(CHUNK_SIZE);
        let cz: i32 = z.div_euclid(CHUNK_SIZE);

        // Chunk block coordinates
        let cbx: i32 = x.rem_euclid(CHUNK_SIZE);
        let cby: i32 = y.rem_euclid(CHUNK_SIZE);
        let cbz: i32 = z.rem_euclid(CHUNK_SIZE);

        if let Some(chunk) = self.get_chunk(cx, cy, cz) {
            return chunk.get_block_from_xyz(cbx, cby, cbz);
        }
        else {
            // If the chunk does not exist / is not found, return air (useful for rendering purpose mainly)
            return BlockInstance::air();
        }
    }

    pub fn get_local_block_from_xyz(&self, lx: i32, ly: i32, lz: i32, cx: i32, cy: i32, cz: i32) -> BlockInstance {
        if !(0..CHUNK_SIZE).contains(&lx)
            || !(0..CHUNK_SIZE).contains(&ly)
            || !(0..CHUNK_SIZE).contains(&lz) {
            return self.get_block_from_xyz(
                lx + cx * CHUNK_SIZE,
                ly + cy * CHUNK_SIZE,
                lz + cz * CHUNK_SIZE
            );   
        }
        
        if let Some(chunk) = self.get_chunk(cx, cy, cz) {
            return chunk.get_block_from_xyz(lx, ly, lz);
        }
        else {
            return BlockInstance::air();
        }
    }
}