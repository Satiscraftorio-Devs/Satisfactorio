use lazy_static::lazy_static;

lazy_static! {
    pub static ref GAME_STATE: GameState = GameState::new();
}

use serde::{Deserialize, Serialize};
use shared::network::messages::{Position, Rotation};
use shared::world::data::chunk::Chunk;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: u64,
    pub username: String,
    pub position: Position,
    pub rotation: Rotation,
}

#[derive(Clone)]
pub struct CachedChunk {
    pub chunk: Chunk,
    pub checksum: [u8; 2],
}

#[derive(Clone)]
pub struct GameState {
    inner: Arc<RwLock<GameStateInner>>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(GameStateInner::new())),
        }
    }

    pub fn init_random_seed(&self) {
        let seed = rand::random::<u32>();
        self.inner.write().unwrap().set_seed(seed);
    }

    pub fn init_seed(&self, seed: u32) {
        self.inner.write().unwrap().set_seed(seed);
    }

    pub fn get_seed(&self) -> u32 {
        self.inner.read().unwrap().get_seed()
    }

    pub fn get_all_players_vec(&self) -> Option<Vec<Player>> {
        self.inner.read().unwrap().get_players_vec()
    }

    pub fn add_player(&self, id: u64, username: String) {
        self.inner.write().unwrap().add_player(id, username);
    }

    pub fn remove_player(&self, id: &u64) -> Option<Player> {
        self.inner.write().unwrap().remove_player(id)
    }

    pub fn update_player_position(&self, id: u64, position: Position, rotation: Rotation) {
        self.inner.write().unwrap().update_player_position(id, position, rotation);
    }

    pub fn cache_chunk(&self, x: i32, y: i32, z: i32, chunk: Chunk) {
        let checksum = chunk.compute_checksum();
        self.inner.write().unwrap().cache_chunk(x, y, z, chunk, checksum);
    }

    pub fn get_cached_chunk(&self, x: i32, y: i32, z: i32) -> Option<Chunk> {
        self.inner.read().unwrap().get_cached_chunk(x, y, z).map(|c| c.chunk.clone())
    }

    pub fn get_cached_checksum(&self, x: i32, y: i32, z: i32) -> Option<[u8; 2]> {
        self.inner.read().unwrap().get_cached_checksum(x, y, z)
    }
}

pub struct GameStateInner {
    pub seed: u32,
    pub players: HashMap<u64, Player>,
    pub chunk_cache: HashMap<(i32, i32, i32), CachedChunk>,
}

impl Default for GameStateInner {
    fn default() -> Self {
        Self::new()
    }
}

impl GameStateInner {
    pub fn new() -> Self {
        Self {
            seed: 0,
            players: HashMap::new(),
            chunk_cache: HashMap::new(),
        }
    }

    pub fn set_seed(&mut self, seed: u32) {
        self.seed = seed;
    }

    pub fn get_seed(&self) -> u32 {
        self.seed
    }

    pub fn add_player(&mut self, id: u64, username: String) {
        let player = Player {
            id,
            username,
            position: Position { x: 0.0, y: 64.0, z: 0.0 },
            rotation: Rotation { x: 0.0, y: 0.0 },
        };
        self.players.insert(id, player);
    }

    pub fn remove_player(&mut self, id: &u64) -> Option<Player> {
        self.players.remove(id)
    }

    pub fn get_player(&self, id: &u64) -> Option<&Player> {
        self.players.get(id)
    }

    pub fn get_players_vec(&self) -> Option<Vec<Player>> {
        Some(self.players.values().cloned().collect())
    }

    pub fn update_player_position(&mut self, id: u64, position: Position, rotation: Rotation) {
        if let Some(player) = self.players.get_mut(&id) {
            player.position = position;
            player.rotation = rotation;
        }
    }

    pub fn cache_chunk(&mut self, x: i32, y: i32, z: i32, chunk: Chunk, checksum: [u8; 2]) {
        self.chunk_cache.insert((x, y, z), CachedChunk { chunk, checksum });
    }

    pub fn get_cached_chunk(&self, x: i32, y: i32, z: i32) -> Option<&CachedChunk> {
        self.chunk_cache.get(&(x, y, z))
    }

    pub fn get_cached_checksum(&self, x: i32, y: i32, z: i32) -> Option<[u8; 2]> {
        self.chunk_cache.get(&(x, y, z)).map(|c| c.checksum)
    }
}
