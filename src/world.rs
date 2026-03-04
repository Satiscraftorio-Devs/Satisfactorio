use std::collections::HashMap;

use cgmath::num_traits::{Euclid, ToPrimitive};

use crate::player::Player;

pub const CHUNK_SIZE: i32 = 32;
pub const CHUNK_SIZE_SQR: i32 = CHUNK_SIZE * CHUNK_SIZE;
pub const CHUNK_BLOCK_NUMBER: usize = (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize;
pub const LAST_CHUNK_BLOCK_INDEX: i32 = CHUNK_SIZE - 1;

#[derive(Clone)]
pub struct Block {
    pub id: i32
}

pub struct Chunk {
    blocks: [i32; CHUNK_BLOCK_NUMBER],
}

pub struct World {
    chunks: HashMap<(i32, i32, i32), Chunk>,
}

impl Block {
    pub fn new(id: i32) -> Block {
        return Block {
            id: id
        };
    }

    pub fn air() -> Block {
        return Block {
            id: 0,
        };
    }

    pub fn is_air(&self) -> bool {
        return self.id == Block::air().id;
    }
}

impl Chunk {
    pub fn generate(cx: i32, cy: i32, cz: i32) -> Chunk {
        let mut chunk = Chunk { blocks: [Block::air().id; CHUNK_BLOCK_NUMBER] };
        let block = Block::new(1);

        // On génère pour l'instant un flat sur la couche y: 0 avec un id bidon pour qu'on ne le mélange pas à l'air et qu'on crée des blocs solides et visibles
        // Pour l'instant on s'en fout des cos du chunk, ça servira plus tard pour des trucs style biomes, température, etc

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                // Todo: génération aléatoire
                chunk.set_block_from_xyz(x, 0, z, block.clone());
            }
        }
        
        return chunk;
    }

    /// Abstraction of `get_block_from_i` but with components.
    /// 
    /// Prefer using `get_block_from_i` whenever possible, as it saves computing power and time. 
    pub fn get_block_from_xyz(&self, x: i32, y: i32, z: i32) -> Block {
        return self.get_block_from_i((x + y * CHUNK_SIZE + z * CHUNK_SIZE_SQR) as usize);
    }

    pub fn get_block_from_i(&self, i: usize) -> Block {
        return Block {
            id: self.blocks[i] as i32,
        };
    }

    /// Abstraction of `set_block_from_i` but with components.
    /// 
    /// Prefer using `set_block_from_i` whenever possible, as it saves computing power and time. 
    pub fn set_block_from_xyz(&mut self, x: i32, y: i32, z: i32, block: Block) {
        self.set_block_from_i((x + y * CHUNK_SIZE + z * CHUNK_SIZE_SQR) as usize, block);
    }

    pub fn set_block_from_i(&mut self, i: usize, block: Block) {
        self.blocks[i] = block.id;
    }
}

impl World {
    pub fn new() -> World {
        return World {
            chunks: HashMap::new()
        };
    }

    pub fn get_chunk(&self, cx: i32, cy: i32, cz: i32) -> Option<&Chunk> {
        return self.chunks.get(&(cx, cy, cz));
    }

    pub fn set_chunk(&mut self, cx: i32, cy: i32, cz: i32, chunk: Chunk) {
        self.chunks.insert((cx, cy, cz), chunk);
    }

    pub fn get_player_rendered_chunks(&self, player: &Player) -> Vec<(&Chunk, i32, i32, i32)> {
        let [min_cx, max_cx, min_cy, max_cy, min_cz, max_cz] = player.get_rendered_chunk_range();
        let chunk_number = player.get_rendered_chunk_number();

        let mut chunks: Vec<(&Chunk, i32, i32, i32)> = Vec::new();
        chunks.reserve_exact(chunk_number as usize);

        for x in min_cx..=max_cx {
            for y in min_cy..=max_cy {
                for z in min_cz..=max_cz {
                    if let Some(chunk) = self.get_chunk(x, y, z) {
                        chunks.push((chunk, x, y, z));
                    }
                }
            }
        }

        return chunks;
    }

    pub fn get_block_from_xyz(&self, x: i32, y: i32, z: i32) -> Block {
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
            return Block::air();
        }
    }

    pub fn get_local_block_from_xyz(&self, lx: i32, ly: i32, lz: i32, cx: i32, cy: i32, cz: i32) -> Block {
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
            return Block::air();
        }
    }
}