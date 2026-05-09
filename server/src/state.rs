use crate::player::PlayerRegistry;
use crate::world::WorldState;
use shared::network::messages::{Position, Rotation};
use std::sync::RwLock;

pub struct AppState {
    inner: RwLock<ServerState>,
}

struct ServerState {
    world: WorldState,
    players: PlayerRegistry,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(ServerState {
                world: WorldState::new(),
                players: PlayerRegistry::new(),
            }),
        }
    }

    pub fn init_random_seed(&self) {
        let seed = rand::random::<u32>();
        self.inner.write().unwrap().world.set_seed(seed);
    }

    pub fn init_seed(&self, seed: u32) {
        self.inner.write().unwrap().world.set_seed(seed);
    }

    pub fn get_seed(&self) -> u32 {
        self.inner.read().unwrap().world.get_seed()
    }

    pub fn add_player(&self, id: u64, username: String) {
        self.inner.write().unwrap().players.add(id, username);
    }

    pub fn remove_player(&self, id: &u64) {
        let mut state = self.inner.write().unwrap();
        state.players.remove(id);
        let keep = state.players.all_required_chunks();
        state.world.retain_chunks(&keep);
    }

    pub fn get_all_players_vec(&self) -> Option<Vec<crate::player::Player>> {
        self.inner.read().unwrap().players.get_all()
    }

    pub fn set_block(&self, x: i32, y: i32, z: i32, block_id: u32) {
        self.inner.write().unwrap().world.set_block(x, y, z, block_id);
    }

    pub fn update_player_position(&self, id: u64, position: Position, rotation: Rotation) {
        let mut state = self.inner.write().unwrap();

        let chunk_pos = state.players.update_position(id, position, rotation);
        let required = WorldState::get_required_chunks(chunk_pos.0, chunk_pos.1, chunk_pos.2);

        state.players.set_player_chunks(id, required.clone());

        let missing: Vec<_> = required
            .iter()
            .filter(|c| !state.world.world_generated_chunks.contains_key(*c))
            .cloned()
            .collect();
        if !missing.is_empty() {
            state.world.generate_missing(&missing);
        }

        let keep = state.players.all_required_chunks();
        state.world.retain_chunks(&keep);
    }
}
