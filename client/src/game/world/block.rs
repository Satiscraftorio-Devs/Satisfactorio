#[derive(Clone, Copy, PartialEq)]
pub enum BlockType {
    Air = 0,
    Grass = 1,
    Dirt = 2,
    Stone = 3,
}

impl BlockType {
    pub fn from_id(id: u32) -> BlockType {
        match id {
            0 => BlockType::Air,
            1 => BlockType::Grass,
            2 => BlockType::Dirt,
            3 => BlockType::Stone,
            _ => BlockType::Dirt,
        }
    }

    pub fn texture_index(&self) -> u32 {
        match self {
            BlockType::Air => 0,
            BlockType::Grass => 0,
            BlockType::Dirt => 1,
            BlockType::Stone => 2,
        }
    }
}

#[derive(Clone, Copy)]
pub struct BlockInstance {
    pub id: u32,
}

impl BlockInstance {
    pub fn new(id: u32) -> BlockInstance {
        return BlockInstance { id: id };
    }

    pub fn air() -> BlockInstance {
        return BlockInstance { id: 0 };
    }

    pub fn is_air(&self) -> bool {
        return self.id == BlockInstance::air().id;
    }

    pub fn is_solid(&self) -> bool {
        return self.id != BlockInstance::air().id;
    }

    pub fn block_type(&self) -> BlockType {
        BlockType::from_id(self.id)
    }

    pub fn texture_index(&self) -> u32 {
        self.block_type().texture_index()
    }
}
