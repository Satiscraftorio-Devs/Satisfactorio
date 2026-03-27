use std::{collections::HashMap, sync::Arc};

use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use wgpu::Device;

use crate::{engine::render::mesh::chunk::ChunkMesh, game::world::world::World};

pub struct WorldMesh {
    pub meshes: HashMap<(i32, i32, i32), Arc<ChunkMesh>>,
}

impl WorldMesh {
    pub fn new() -> WorldMesh {
        return WorldMesh {
            meshes: HashMap::new(),
        };
    }

    pub fn update(
        &mut self,
        device: &Device,
        world: &mut World,
        chunks_to_rebuild: &[(i32, i32, i32)],
    ) {
        let mesh_keys: Vec<_> = self.meshes.keys().cloned().collect();
        for key in mesh_keys {
            if !world.get_chunk_data(key.0, key.1, key.2).is_some() {
                self.meshes.remove(&key);
            }
        }

        if chunks_to_rebuild.is_empty() {
            return;
        }

        let rebuilt_chunks: Vec<_> = chunks_to_rebuild
            .par_iter()
            .filter_map(|&(cx, cy, cz)| {
                if let Some(chunk_data) = world.get_chunk_data(cx, cy, cz) {
                    let mut mesh = ChunkMesh::new();
                    mesh.make_greedy(&chunk_data.chunk, world, device, cx, cy, cz);
                    return Some(((cx, cy, cz), Arc::new(mesh)));
                }
                None
            })
            .collect();

        for (key, mesh) in rebuilt_chunks {
            world.mark_chunk_clean(key.0, key.1, key.2);
            self.meshes.insert(key, mesh);
        }
    }
}
