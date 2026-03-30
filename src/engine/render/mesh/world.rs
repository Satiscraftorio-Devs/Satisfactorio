use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    engine::render::{mesh::chunk::ChunkMesh, render::Renderer},
    game::{player::player::Player, world::world::World},
};

pub struct WorldMesh {
    pub meshes: HashMap<(i32, i32, i32), Arc<ChunkMesh>>,
}

impl WorldMesh {
    pub fn new() -> WorldMesh {
        return WorldMesh { meshes: HashMap::new() };
    }

    pub fn update(&mut self, renderer: &mut Renderer, world: &World, _player: &Player, chunks_to_rebuild: &[(i32, i32, i32)]) {
        if chunks_to_rebuild.is_empty() {
            return;
        }

        let shared_rm = Arc::new(Mutex::new(renderer));

        for &(cx, cy, cz) in chunks_to_rebuild {
            if let Some(chunk_data) = world.get_chunk_data(cx, cy, cz) {
                let key = (cx, cy, cz);
                let mut mesh = ChunkMesh::new();
                {
                    let mut rm = shared_rm.lock().unwrap();
                    mesh.make_greedy(&chunk_data.chunk, world, &mut *rm, cx, cy, cz);
                }
                self.meshes.insert(key, Arc::new(mesh));
            }
        }
    }
}
