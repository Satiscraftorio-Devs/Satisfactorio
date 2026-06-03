pub struct TextureID {
    array: usize,
    depth: u16,
}

impl TextureID {
    pub fn new(array: usize, depth: u16) -> Self {
        Self { array, depth }
    }

    pub fn array(&self) -> usize {
        self.array
    }

    pub fn depth(&self) -> u16 {
        self.depth
    }
}
