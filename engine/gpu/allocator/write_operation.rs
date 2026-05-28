pub type MeshId = u32;

pub struct WriteOperation {
    pub mesh_id: MeshId,
    pub offset: usize,
    pub len: usize,
    pub arena_offset: usize,
}
