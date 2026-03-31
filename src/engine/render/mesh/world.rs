use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    engine::render::{mesh::chunk::ChunkMesh, render::Renderer},
    game::{player::player::Player, world::world::World},
};

pub struct WorldMesh {
    pub meshes: HashMap<(i32, i32, i32), ChunkMesh>,
}

impl WorldMesh {
    pub fn new() -> WorldMesh {
        return WorldMesh { meshes: HashMap::new() };
    }

    pub fn update(&mut self, renderer: &mut Renderer, world: &World, player: &Player) {
        let [min_cx, max_cx, min_cy, max_cy, min_cz, max_cz] = player.get_rendered_chunk_range();
        let mut needed_rendered_keys: Vec<(i32, i32, i32)> = Vec::new();

        for x in min_cx..=max_cx {
            for y in min_cy..=max_cy {
                for z in min_cz..=max_cz {
                    needed_rendered_keys.push((x, y, z));
                }
            }
        }

        let shared_rm = Arc::new(Mutex::new(renderer));

        for &(cx, cy, cz) in needed_rendered_keys.iter() {
            if let Some(chunk_data) = world.get_chunk_data(cx, cy, cz) {
                let key = (cx, cy, cz);
                if let Some(mesh) = self.meshes.get_mut(&key) {
                    let mut rm = shared_rm.lock().unwrap();
                    mesh.make_greedy(&chunk_data.chunk, world, &mut *rm, cx, cy, cz);
                }
                else
                {
                    let mut mesh = ChunkMesh::new();
                    let mut rm = shared_rm.lock().unwrap();
                    mesh.make_greedy(&chunk_data.chunk, world, &mut *rm, cx, cy, cz);
                    self.meshes.insert(key, mesh);
                }
            }
        }
    }
}
