use crate::game::world::{
    block::BlockInstance,
    chunk::{Chunk, CHUNK_SIZE, LAST_CHUNK_AXIS_INDEX},
    world::World,
};

pub const PADDED_CHUNK_SIZE: i32 = CHUNK_SIZE + 2;
pub const PADDED_CHUNK_SIZE_DOUBLE: usize = (PADDED_CHUNK_SIZE * 2) as usize;
pub const PADDED_CHUNK_SIZE_SQR: i32 = PADDED_CHUNK_SIZE * PADDED_CHUNK_SIZE;
pub const PADDED_CHUNK_BLOCK_NUMBER: usize = (PADDED_CHUNK_SIZE * PADDED_CHUNK_SIZE * PADDED_CHUNK_SIZE) as usize;
pub const FIRST_PADDED_CHUNK_CENTER_INDEX: i32 = 1;
pub const LAST_PADDED_CHUNK_CENTER_INDEX: i32 = PADDED_CHUNK_SIZE - 2;
pub const FIRST_PADDED_CHUNK_AXIS_INDEX: i32 = 0;
pub const LAST_PADDED_CHUNK_AXIS_INDEX: i32 = PADDED_CHUNK_SIZE - 1;
pub const LAST_PADDED_CHUNK_AXIS_INDEX_USIZE: usize = LAST_PADDED_CHUNK_AXIS_INDEX as usize;

pub struct PaddedChunk {
    blocks: [BlockInstance; PADDED_CHUNK_BLOCK_NUMBER],
}

impl PaddedChunk {
    pub fn empty() -> PaddedChunk {
        return PaddedChunk {
            blocks: [BlockInstance::air(); PADDED_CHUNK_BLOCK_NUMBER],
        };
    }

    pub fn new(chunk: &Chunk, world: &World) -> PaddedChunk {
        let mut padded_chunk = PaddedChunk::empty();

        let mut src_i = 0usize;
        let mut dst_i = (1 + PADDED_CHUNK_SIZE + PADDED_CHUNK_SIZE_SQR) as usize; // (1,1,1)

        // Heart
        for _z in 0..CHUNK_SIZE {
            for _y in 0..CHUNK_SIZE {
                for _x in 0..CHUNK_SIZE {
                    padded_chunk.blocks[dst_i] = chunk.get_block_from_i(src_i);

                    src_i += 1;
                    dst_i += 1;
                }

                // fin ligne X → sauter bordure droite + gauche
                dst_i += 2;
            }

            // fin plan Y → sauter 2 lignes complètes (haut/bas)
            dst_i += PADDED_CHUNK_SIZE_DOUBLE;
        }

        // Edges
        padded_chunk.fill_edges(
            world.get_chunk(chunk.x - 1, chunk.y, chunk.z),
            world.get_chunk(chunk.x + 1, chunk.y, chunk.z),
            world.get_chunk(chunk.x, chunk.y - 1, chunk.z),
            world.get_chunk(chunk.x, chunk.y + 1, chunk.z),
            world.get_chunk(chunk.x, chunk.y, chunk.z - 1),
            world.get_chunk(chunk.x, chunk.y, chunk.z + 1),
        );

        // Corners (VERTICAL VIEW)
        // Bottom face

        // TODO
        // padded_chunk.set_block_from_xyz(0, 0, 0, GET_BLOCK);

        return padded_chunk;
    }

    /// Abstraction of `get_block_from_i` but restricted to the actual chunk it represents, and with components.
    ///
    /// Prefer using `get_block_from_i` whenever possible, as it saves computing power and time.
    #[inline(always)]
    pub fn get_block_from_chunk_xyz(&self, x: i32, y: i32, z: i32) -> BlockInstance {
        // Clamp coordinates to valid range and return air for out-of-bounds
        let cx = x.clamp(-1, CHUNK_SIZE);
        let cy = y.clamp(-1, CHUNK_SIZE);
        let cz = z.clamp(-1, CHUNK_SIZE);

        // Check if original coordinates were out of bounds - return air if so
        if x != cx || y != cy || z != cz {
            // println!("out of bounds : {} {} ; {} {} ; {} ", cx, x, cy, y, cz, z);
            return BlockInstance::air();
        }

        return self.get_block_from_i(((x + 1) + (y + 1) * PADDED_CHUNK_SIZE + (z + 1) * PADDED_CHUNK_SIZE_SQR) as usize);
    }

    /// Abstraction of `get_block_from_i` but with components.
    ///
    /// Prefer using `get_block_from_i` whenever possible, as it saves computing power and time.
    #[inline(always)]
    pub fn get_block_from_xyz(&self, x: i32, y: i32, z: i32) -> BlockInstance {
        if x < 0 || y < 0 || z < 0 || x >= PADDED_CHUNK_SIZE || y >= PADDED_CHUNK_SIZE || z >= PADDED_CHUNK_SIZE {
            return BlockInstance::air();
        }
        return self.get_block_from_i((x + y * PADDED_CHUNK_SIZE + z * PADDED_CHUNK_SIZE_SQR) as usize);
    }

    #[inline(always)]
    pub fn get_block_from_i(&self, i: usize) -> BlockInstance {
        return self.blocks[i];
    }

    /// Abstraction of `set_block_from_i` but with components.
    ///
    /// Prefer using `set_block_from_i` whenever possible, as it saves computing power and time.
    #[inline(always)]
    fn set_block_from_xyz(&mut self, x: i32, y: i32, z: i32, block: BlockInstance) {
        self.set_block_from_i((x + y * PADDED_CHUNK_SIZE + z * PADDED_CHUNK_SIZE_SQR) as usize, block);
    }

    #[inline(always)]
    fn set_block_from_i(&mut self, i: usize, block: BlockInstance) {
        self.blocks[i] = block;
    }

    pub fn fill_neg_x(&mut self, chunk: &Chunk) {
        for y in FIRST_PADDED_CHUNK_CENTER_INDEX..=LAST_PADDED_CHUNK_CENTER_INDEX {
            for z in FIRST_PADDED_CHUNK_CENTER_INDEX..=LAST_PADDED_CHUNK_CENTER_INDEX {
                self.set_block_from_xyz(0, y, z, chunk.get_block_from_xyz(LAST_CHUNK_AXIS_INDEX, y - 1, z - 1));
            }
        }
    }

    pub fn fill_pos_x(&mut self, chunk: &Chunk) {
        for y in FIRST_PADDED_CHUNK_CENTER_INDEX..=LAST_PADDED_CHUNK_CENTER_INDEX {
            for z in FIRST_PADDED_CHUNK_CENTER_INDEX..=LAST_PADDED_CHUNK_CENTER_INDEX {
                self.set_block_from_xyz(LAST_PADDED_CHUNK_AXIS_INDEX, y, z, chunk.get_block_from_xyz(0, y - 1, z - 1));
            }
        }
    }

    pub fn fill_neg_y(&mut self, chunk: &Chunk) {
        for x in FIRST_PADDED_CHUNK_CENTER_INDEX..=LAST_PADDED_CHUNK_CENTER_INDEX {
            for z in FIRST_PADDED_CHUNK_CENTER_INDEX..=LAST_PADDED_CHUNK_CENTER_INDEX {
                self.set_block_from_xyz(x, 0, z, chunk.get_block_from_xyz(x - 1, LAST_CHUNK_AXIS_INDEX, z - 1));
            }
        }
    }

    pub fn fill_pos_y(&mut self, chunk: &Chunk) {
        for x in FIRST_PADDED_CHUNK_CENTER_INDEX..=LAST_PADDED_CHUNK_CENTER_INDEX {
            for z in FIRST_PADDED_CHUNK_CENTER_INDEX..=LAST_PADDED_CHUNK_CENTER_INDEX {
                self.set_block_from_xyz(x, LAST_PADDED_CHUNK_AXIS_INDEX, z, chunk.get_block_from_xyz(x - 1, 0, z - 1));
            }
        }
    }

    pub fn fill_neg_z(&mut self, chunk: &Chunk) {
        for x in FIRST_PADDED_CHUNK_CENTER_INDEX..=LAST_PADDED_CHUNK_CENTER_INDEX {
            for y in FIRST_PADDED_CHUNK_CENTER_INDEX..=LAST_PADDED_CHUNK_CENTER_INDEX {
                self.set_block_from_xyz(x, y, 0, chunk.get_block_from_xyz(x - 1, y - 1, LAST_CHUNK_AXIS_INDEX));
            }
        }
    }

    pub fn fill_pos_z(&mut self, chunk: &Chunk) {
        for x in FIRST_PADDED_CHUNK_CENTER_INDEX..=LAST_PADDED_CHUNK_CENTER_INDEX {
            for y in FIRST_PADDED_CHUNK_CENTER_INDEX..=LAST_PADDED_CHUNK_CENTER_INDEX {
                self.set_block_from_xyz(x, y, LAST_PADDED_CHUNK_AXIS_INDEX, chunk.get_block_from_xyz(x - 1, y - 1, 0));
            }
        }
    }

    pub fn fill_edges(
        &mut self,
        neg_x: Option<&Chunk>,
        pos_x: Option<&Chunk>,
        neg_y: Option<&Chunk>,
        pos_y: Option<&Chunk>,
        neg_z: Option<&Chunk>,
        pos_z: Option<&Chunk>,
    ) {
        if let Some(neg_x) = neg_x {
            self.fill_neg_x(neg_x);
        } else {
            self.fill_neg_x_as_air();
        }
        if let Some(pos_x) = pos_x {
            self.fill_pos_x(pos_x);
        } else {
            self.fill_pos_x_as_air();
        }
        if let Some(neg_y) = neg_y {
            self.fill_neg_y(neg_y);
        } else {
            self.fill_neg_y_as_air();
        }
        if let Some(pos_y) = pos_y {
            self.fill_pos_y(pos_y);
        } else {
            self.fill_pos_y_as_air();
        }
        if let Some(neg_z) = neg_z {
            self.fill_neg_z(neg_z);
        } else {
            self.fill_neg_z_as_air();
        }
        if let Some(pos_z) = pos_z {
            self.fill_pos_z(pos_z);
        } else {
            self.fill_pos_z_as_air();
        }
    }

    fn fill_neg_x_as_air(&mut self) {
        for y in FIRST_PADDED_CHUNK_CENTER_INDEX..=LAST_PADDED_CHUNK_CENTER_INDEX {
            for z in FIRST_PADDED_CHUNK_CENTER_INDEX..=LAST_PADDED_CHUNK_CENTER_INDEX {
                self.set_block_from_xyz(0, y, z, BlockInstance::air());
            }
        }
    }

    fn fill_pos_x_as_air(&mut self) {
        for y in FIRST_PADDED_CHUNK_CENTER_INDEX..=LAST_PADDED_CHUNK_CENTER_INDEX {
            for z in FIRST_PADDED_CHUNK_CENTER_INDEX..=LAST_PADDED_CHUNK_CENTER_INDEX {
                self.set_block_from_xyz(LAST_PADDED_CHUNK_AXIS_INDEX, y, z, BlockInstance::air());
            }
        }
    }

    fn fill_neg_y_as_air(&mut self) {
        for x in FIRST_PADDED_CHUNK_CENTER_INDEX..=LAST_PADDED_CHUNK_CENTER_INDEX {
            for z in FIRST_PADDED_CHUNK_CENTER_INDEX..=LAST_PADDED_CHUNK_CENTER_INDEX {
                self.set_block_from_xyz(x, 0, z, BlockInstance::air());
            }
        }
    }

    fn fill_pos_y_as_air(&mut self) {
        for x in FIRST_PADDED_CHUNK_CENTER_INDEX..=LAST_PADDED_CHUNK_CENTER_INDEX {
            for z in FIRST_PADDED_CHUNK_CENTER_INDEX..=LAST_PADDED_CHUNK_CENTER_INDEX {
                self.set_block_from_xyz(x, LAST_PADDED_CHUNK_AXIS_INDEX, z, BlockInstance::air());
            }
        }
    }

    fn fill_neg_z_as_air(&mut self) {
        for x in FIRST_PADDED_CHUNK_CENTER_INDEX..=LAST_PADDED_CHUNK_CENTER_INDEX {
            for y in FIRST_PADDED_CHUNK_CENTER_INDEX..=LAST_PADDED_CHUNK_CENTER_INDEX {
                self.set_block_from_xyz(x, y, 0, BlockInstance::air());
            }
        }
    }

    fn fill_pos_z_as_air(&mut self) {
        for x in FIRST_PADDED_CHUNK_CENTER_INDEX..=LAST_PADDED_CHUNK_CENTER_INDEX {
            for y in FIRST_PADDED_CHUNK_CENTER_INDEX..=LAST_PADDED_CHUNK_CENTER_INDEX {
                self.set_block_from_xyz(x, y, LAST_PADDED_CHUNK_AXIS_INDEX, BlockInstance::air());
            }
        }
    }
}
