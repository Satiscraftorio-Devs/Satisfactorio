use crate::{
    engine::render::{manager::RenderManager, texture::TextureArrayIndex},
    game::api::texture_loader::TextureLoader,
};
use shared::world::{
    data::{
        block::{BlockInstance, BlockManager},
        chunk::{Chunk, ChunkData, ChunkState, CHUNK_SIZE},
    },
    generation::chunk_generator::ChunkGenerator,
};
use shared::{constants::DIRECT_NORMALS_3D, world::data::block::BlockData};
use shared::{constants::MAX_CHUNKS_IN_QUEUE, log_err_client};
use std::{
    cmp::max,
    collections::{HashMap, VecDeque},
    sync::{Arc, RwLock},
};

use crate::{
    // engine::render::mesh::manager::RenderManager,
    game::{player::player::Player, render::meshing::world::WorldMesh},
};

#[derive(Clone)]
pub struct MeshSnapshot {
    pub main: Arc<Chunk>,
    pub neg_x: Option<Arc<Chunk>>,
    pub pos_x: Option<Arc<Chunk>>,
    pub neg_y: Option<Arc<Chunk>>,
    pub pos_y: Option<Arc<Chunk>>,
    pub neg_z: Option<Arc<Chunk>>,
    pub pos_z: Option<Arc<Chunk>>,
}

pub struct World {
    seed: u32,
    chunks: HashMap<(i32, i32, i32), ChunkData>,
    chunk_generator: ChunkGenerator,
    block_manager: Arc<RwLock<BlockManager>>,
    ready_to_mesh: VecDeque<(i32, i32, i32)>,
}

impl World {
    pub fn new(seed: u32) -> World {
        let block_manager = Arc::new(RwLock::new(BlockManager::new()));

        let worker_count = max((num_cpus::get() as f32 / 2.0).floor() as usize, 1);
        let chunk_generator =
            ChunkGenerator::with_max_pending(worker_count, Arc::clone(&block_manager), seed, MAX_CHUNKS_IN_QUEUE as usize);

        return World {
            seed: seed,
            chunks: HashMap::new(),
            chunk_generator: chunk_generator,
            block_manager: block_manager,
            ready_to_mesh: VecDeque::new(),
        };
    }

    pub fn init(&mut self, texture_loader: &mut TextureLoader, player: &Player) {
        {
            let mut block_manager = self.block_manager.write().unwrap();

            let blocks = [
                (BlockData::new("air"), ""),
                (BlockData::new("stone"), "assets/images/stone.png"),
                (BlockData::new("dirt"), "assets/images/dirt.png"),
                (BlockData::new("grass"), "assets/images/grass.png"),
            ];

            for mut values in blocks {
                if let Ok(tex_id) = texture_loader.register(values.1.to_string(), TextureArrayIndex::Opaque) {
                    values.0.texture_index = Some(tex_id.depth() as u32);
                };
                block_manager.register(values.0);
            }
        }

        self.generate_missing_chunks(player);
    }

    pub fn update(&mut self, render_manager: &mut RenderManager, world_mesh: &mut WorldMesh, player: &Player) {
        if player.state.cpos.has_changed() || self.chunks.is_empty() {
            self.clean_chunks(render_manager, world_mesh, player);
            self.generate_missing_chunks(player);
        }

        self.compute_generated_chunks();
    }

    fn clean_chunks(&mut self, render_manager: &mut RenderManager, world_mesh: &mut WorldMesh, player: &Player) {
        // Maths
        let radii_h_squared = (player.state.horizontal_render_distance * player.state.horizontal_render_distance) as i32;
        let radii_v_squared = (player.state.vertical_render_distance * player.state.vertical_render_distance) as i32;

        // Coordonnées du chunk où se trouve le joueur (entiers)
        let cpos = player.get_cpos().into();

        self.chunks.retain(|key, _| {
            // On garde les chunks qui sont à portée du joueur
            if is_chunk_in_range(key, &cpos, radii_h_squared, radii_v_squared) {
                return true;
            }
            // On supprime proprement ceux qui ne le sont plus
            let Some(mesh) = world_mesh.meshes.remove(key) else {
                return false;
            };
            let Some(id) = mesh.id else {
                return false;
            };
            let result = render_manager.mesh_manager.free_data(id);
            match result {
                Ok(_) => {}
                Err(err) => {
                    let (x, y, z) = key;
                    log_err_client!(
                        "World: an error occured while trying to free chunk (x: {} y: {} z: {})'s mesh with id {}.\nError: {}",
                        x,
                        y,
                        z,
                        id,
                        err
                    )
                }
            }
            false
        });
    }

    fn generate_missing_chunks(&mut self, player: &Player) {
        // Si la file d'attente est pleine, ça ne sert à rien d'essayer de soumettre des demandes
        if self.chunk_generator.is_queue_full() {
            return;
        }

        // Identifier les chunks manquants
        let mut missing_keys: Vec<(i32, i32, i32)> = {
            let mut needed_keys = player.get_simulation_chunk_keys();
            needed_keys.retain(|key| !self.chunks.contains_key(key));
            needed_keys
        };

        // Coordonnées joueur (entiers)
        let (px, py, pz) = player.get_pos().map(|coord| coord.floor() as i32).into();

        // Trier les demandes de chunks manquants en fonction de leur distance au joueur (proche = prioritaire)
        missing_keys.sort_by_key(|chunk| {
            // Chaque coordonnée représente la position du chunk dans le monde, à l'échelle d'un bloc, et centré sur le joueur
            let x = chunk.0 * CHUNK_SIZE - px;
            let y = chunk.1 * CHUNK_SIZE - py;
            let z = chunk.2 * CHUNK_SIZE - pz;

            // Distance au carrée
            x * x + y * y + z * z
        });

        // On soumet les clés des chunks manquants pour les faire générer
        for (cx, cy, cz) in missing_keys {
            let result = self.chunk_generator.request(cx, cy, cz);
            match result {
                Ok(_) => {}
                Err(_) => {
                    // La file d'attente est pleine, on arrête ici pour l'instant
                    break;
                }
            }
        }
    }

    fn compute_generated_chunks(&mut self) {
        while let Some(result) = self.chunk_generator.try_recv() {
            let (cx, cy, cz, chunk_with_checksum) = result.output;
            let key = (cx, cy, cz);

            let mut chunk = chunk_with_checksum.chunk_data;
            chunk.is_dirty = false;
            self.chunks.insert(key, chunk);
            if self.are_all_neighbors_ready(cx, cy, cz) && !self.ready_to_mesh.contains(&key) {
                self.ready_to_mesh.push_back(key);
            }

            for (dx, dy, dz) in DIRECT_NORMALS_3D {
                let key = (cx + dx, cy + dy, cz + dz);
                if self.are_all_neighbors_ready(key.0, key.1, key.2) && !self.ready_to_mesh.contains(&key) {
                    self.ready_to_mesh.push_back(key);
                }
            }
        }
    }

    pub fn ready_to_mesh(&self) -> &VecDeque<(i32, i32, i32)> {
        &self.ready_to_mesh
    }

    pub fn set_ready_to_mesh(&mut self, new: VecDeque<(i32, i32, i32)>) {
        self.ready_to_mesh = new;
    }

    pub fn is_dirty_at(&self, cx: i32, cy: i32, cz: i32) -> bool {
        self.chunks.get(&(cx, cy, cz)).map_or(false, |chunk| chunk.is_dirty)
    }

    pub fn get_mesh_snapshot(&self, cx: i32, cy: i32, cz: i32) -> MeshSnapshot {
        MeshSnapshot {
            main: Arc::clone(&self.chunks.get(&(cx, cy, cz)).unwrap().chunk),
            neg_x: self.chunks.get(&(cx - 1, cy, cz)).map(|d| Arc::clone(&d.chunk)),
            neg_y: self.chunks.get(&(cx, cy - 1, cz)).map(|d| Arc::clone(&d.chunk)),
            neg_z: self.chunks.get(&(cx, cy, cz - 1)).map(|d| Arc::clone(&d.chunk)),
            pos_x: self.chunks.get(&(cx + 1, cy, cz)).map(|d| Arc::clone(&d.chunk)),
            pos_y: self.chunks.get(&(cx, cy + 1, cz)).map(|d| Arc::clone(&d.chunk)),
            pos_z: self.chunks.get(&(cx, cy, cz + 1)).map(|d| Arc::clone(&d.chunk)),
        }
    }

    #[inline(always)]
    pub fn get_chunk_data(&self, cx: i32, cy: i32, cz: i32) -> Option<&ChunkData> {
        return self.chunks.get(&(cx, cy, cz));
    }

    #[inline(always)]
    pub fn get_chunk_data_mut(&mut self, cx: i32, cy: i32, cz: i32) -> Option<&mut ChunkData> {
        return self.chunks.get_mut(&(cx, cy, cz));
    }

    #[inline(always)]
    pub fn get_chunk(&self, cx: i32, cy: i32, cz: i32) -> Option<&Chunk> {
        return self.chunks.get(&(cx, cy, cz)).map(|d| d.chunk.as_ref());
    }

    #[inline(always)]
    pub fn get_chunk_mut(&mut self, cx: i32, cy: i32, cz: i32) -> Option<&mut ChunkData> {
        return self.chunks.get_mut(&(cx, cy, cz));
    }

    pub fn set_chunk(&mut self, cx: i32, cy: i32, cz: i32, chunk: Chunk) {
        self.chunks.insert((cx, cy, cz), ChunkData::new(chunk));
    }

    pub fn chunk_coords_from_block(x: i32, y: i32, z: i32) -> (i32, i32, i32) {
        (x.div_euclid(CHUNK_SIZE), y.div_euclid(CHUNK_SIZE), z.div_euclid(CHUNK_SIZE))
    }

    pub fn local_block_coords(x: i32, y: i32, z: i32) -> (i32, i32, i32) {
        (x.rem_euclid(CHUNK_SIZE), y.rem_euclid(CHUNK_SIZE), z.rem_euclid(CHUNK_SIZE))
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: BlockInstance) -> bool {
        let (cx, cy, cz) = World::chunk_coords_from_block(x, y, z);
        let (lx, ly, lz) = World::local_block_coords(x, y, z);
        let Some(chunk) = self.get_chunk_data_mut(cx, cy, cz) else {
            return false;
        };
        println!("lx: {} ly: {} lz: {}", lx, ly, lz);
        let current_block = chunk.chunk.get_block_from_xyz(lx, ly, lz);
        if current_block == block {
            return false;
        } else {
            Arc::make_mut(&mut chunk.chunk).set_block_from_xyz(lx, ly, lz, block);
            return true;
        }
    }

    fn set_chunk_dirty(&mut self, cpos: &(i32, i32, i32)) {
        if let Some(chunk) = self.chunks.get_mut(cpos) {
            chunk.set_dirty();
        }
    }

    pub fn get_player_rendered_chunks(&self, player: &Player) -> Vec<&Chunk> {
        let [min_cx, max_cx, min_cy, max_cy, min_cz, max_cz] = player.get_rendered_chunk_range();

        let mut chunks: Vec<&Chunk> = Vec::new();

        for x in min_cx..=max_cx {
            for y in min_cy..=max_cy {
                for z in min_cz..=max_cz {
                    if let Some(data) = self.get_chunk_data(x, y, z) {
                        chunks.push(&data.chunk);
                    }
                }
            }
        }

        return chunks;
    }

    pub fn get_dirty_chunks(&self) -> Vec<(i32, i32, i32)> {
        self.chunks.iter().filter(|(_, data)| data.is_dirty).map(|(key, _)| *key).collect()
    }

    pub fn mark_chunk_clean(&mut self, cx: i32, cy: i32, cz: i32) {
        if let Some(data) = self.chunks.get_mut(&(cx, cy, cz)) {
            data.is_dirty = false;
        }
    }

    pub fn get_block_from_xyz(&self, x: i32, y: i32, z: i32) -> BlockInstance {
        let cx: i32 = x.div_euclid(CHUNK_SIZE);
        let cy: i32 = y.div_euclid(CHUNK_SIZE);
        let cz: i32 = z.div_euclid(CHUNK_SIZE);

        let cbx: i32 = x.rem_euclid(CHUNK_SIZE);
        let cby: i32 = y.rem_euclid(CHUNK_SIZE);
        let cbz: i32 = z.rem_euclid(CHUNK_SIZE);

        if let Some(data) = self.get_chunk_data(cx, cy, cz) {
            return data.chunk.get_block_from_xyz(cbx, cby, cbz);
        } else {
            return BlockInstance::air();
        }
    }

    pub fn get_local_block_from_xyz(&self, lx: i32, ly: i32, lz: i32, cx: i32, cy: i32, cz: i32) -> BlockInstance {
        if !(0..CHUNK_SIZE).contains(&lx) || !(0..CHUNK_SIZE).contains(&ly) || !(0..CHUNK_SIZE).contains(&lz) {
            return self.get_block_from_xyz(lx + cx * CHUNK_SIZE, ly + cy * CHUNK_SIZE, lz + cz * CHUNK_SIZE);
        }

        if let Some(data) = self.get_chunk_data(cx, cy, cz) {
            return data.chunk.get_block_from_xyz(lx, ly, lz);
        } else {
            return BlockInstance::air();
        }
    }

    pub fn are_all_neighbors_ready(&self, cx: i32, cy: i32, cz: i32) -> bool {
        // Requires that every direct chunk neighbor's state is Ready.
        // If a neighbor chunk is missing, it is considered blocking to avoid inter-chunks faces.
        for (dx, dy, dz) in DIRECT_NORMALS_3D {
            if let Some(neighbor) = self.chunks.get(&(cx + dx, cy + dy, cz + dz)) {
                if neighbor.state != ChunkState::Ready {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    /// Retourne vrai si aucun chunk n'est chargé
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    pub fn dispose(&mut self) {
        self.chunks.clear();
        // TODO: faire fonctionner -> self.block_manager.dispose();
        self.chunk_generator.dispose();
    }
}

#[inline(always)]
fn is_chunk_in_range(c: &(i32, i32, i32), center: &(i32, i32, i32), radius_h_squared: i32, radius_v_squared: i32) -> bool {
    let dx = c.0 - center.0;
    let dy = c.1 - center.1;
    let dz = c.2 - center.2;

    (dx * dx) / radius_h_squared + (dy * dy) / radius_v_squared + (dz * dz) / radius_h_squared <= 1
}
