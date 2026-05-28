#[derive(Clone)]
pub struct Gap {
    pub position: usize,
    pub length: usize,
}

impl Gap {
    pub fn new(position: usize, length: usize) -> Self {
        Self { position, length }
    }
}
