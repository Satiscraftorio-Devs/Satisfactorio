#[derive(Clone, Copy)]
pub struct BlockInstance {
    pub id: u32
}

impl BlockInstance {
    pub fn new(id: u32) -> BlockInstance {
        return BlockInstance {
            id: id
        };
    }

    pub fn air() -> BlockInstance {
        return BlockInstance {
            id: 0,
        };
    }

    pub fn is_air(&self) -> bool {
        return self.id == BlockInstance::air().id;
    }

    pub fn is_solid(&self) -> bool {
        return self.id != BlockInstance::air().id;
    }
}