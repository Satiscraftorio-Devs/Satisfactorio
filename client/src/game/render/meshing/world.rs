use crate::{
    engine::render::render::Renderer,
    game::{
        player::player::Player,
        render::meshing::chunk::{ChunkMesh, GreedyMeshingProcessor},
        world::world::World,
    },
};
use shared::parallel::{WorkResult, WorkerPool};
use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

pub struct WorldMesh {
    pub meshes: HashMap<(i32, i32, i32), ChunkMesh>,
    mesh_worker: WorkerPool<GreedyMeshingProcessor>,
    pending: HashSet<(i32, i32, i32)>,
}

impl WorldMesh {
    pub fn new() -> WorldMesh {
        WorldMesh {
            meshes: HashMap::new(),
            mesh_worker: WorkerPool::new(num_cpus::get(), ()),
            pending: HashSet::new(),
        }
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

        let _world_mesh_make_start = Instant::now();

        for &(cx, cy, cz) in needed_rendered_keys.iter() {
            let key = (cx, cy, cz);

            if self.pending.contains(&key) {
                continue;
            }

            if let Some(chunk_data) = world.get_chunk_data(cx, cy, cz) {
                let needs_processing = self.meshes.get(&key).map_or(true, |mesh: &ChunkMesh| mesh.is_dirty());

                if needs_processing && world.are_all_neighbors_ready(cx, cy, cz) {
                    let snapshot = world.get_mesh_snapshot(cx, cy, cz);
                    self.pending.insert(key);
                    self.mesh_worker
                        .submit((chunk_data.chunk.clone().into(), snapshot, cx, cy, cz), key);
                }
            }
        }

        while let Some(WorkResult {
            output: vertices_opt,
            coords,
        }) = self.mesh_worker.try_recv()
        {
            self.pending.remove(&coords);

            if let Some(vertices) = vertices_opt {
                if let Some(mesh) = self.meshes.get_mut(&coords) {
                    mesh.make_greedy(vertices, renderer);
                } else {
                    let mut mesh = ChunkMesh::new();
                    mesh.make_greedy(vertices, renderer);
                    self.meshes.insert(coords, mesh);
                }
            }
        }

        // println!("WorldMesh update took {:.3}ms.", world_mesh_make_start.elapsed().as_micros().to_f64().unwrap() / 1_000.0);
    }
}
