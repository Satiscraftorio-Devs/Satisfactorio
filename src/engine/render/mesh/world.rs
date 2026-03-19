use std::{collections::HashMap, sync::{Arc, Mutex}};

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use wgpu::Device;

use crate::{engine::render::{mesh::chunk::ChunkMesh, render::{RenderManager, Renderer}}, game::{player::player::Player, world::world::World}};

pub struct WorldMesh {
    pub meshes: HashMap<(i32, i32, i32), Arc<ChunkMesh>>,
}

impl WorldMesh {
    pub fn new() -> WorldMesh {
        return WorldMesh {
            meshes: HashMap::new(),
        };
    }

    /// Builds simultaneously every single chunk within the player's both horizontal and vertical render distance only if it needs it (if dirty == true).
    pub fn update(&mut self, renderer: &mut Renderer, world: &World, player: &Player) {
        let shared_rm = Arc::new(Mutex::new(renderer));
        self.meshes = world
            .get_player_rendered_chunks(player)
            .into_par_iter()
            .map(|chunk| {
                let key = (chunk.x, chunk.y, chunk.z);

                if let Some(existing) = self.meshes.get(&key) {
                    if existing.is_dirty() {
                        return (key, Arc::clone(existing));
                    }
                }

                let mut mesh = ChunkMesh::new();
                {
                    let mut rm = shared_rm.lock().unwrap();
                    mesh.make_greedy(chunk, world, &mut *rm, key.0, key.1, key.2);
                }
                return (key, Arc::new(mesh));
            })
            .collect();
    }
}
