use serde::{Deserialize, Serialize};
use shared::network::messages::{Position, Rotation};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: u64,
    pub username: String,
    pub position: Position,
    pub rotation: Rotation,
}

pub struct PlayerRegistry {
    players: HashMap<u64, Player>,
    player_chunks: HashMap<u64, HashSet<(i32, i32, i32)>>,
}

impl PlayerRegistry {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            player_chunks: HashMap::new(),
        }
    }

    pub fn add(&mut self, id: u64, username: String) {
        let player = Player {
            id,
            username,
            position: Position { x: 0.0, y: 64.0, z: 0.0 },
            rotation: Rotation { x: 0.0, y: 0.0 },
        };
        self.players.insert(id, player);
    }

    pub fn remove(&mut self, id: &u64) -> Option<Player> {
        self.player_chunks.remove(id);
        self.players.remove(id)
    }

    pub fn get(&self, id: &u64) -> Option<&Player> {
        self.players.get(id)
    }

    pub fn get_all(&self) -> Option<Vec<Player>> {
        Some(self.players.values().cloned().collect())
    }

    pub fn update_position(&mut self, id: u64, position: Position, rotation: Rotation) -> (i32, i32, i32) {
        if let Some(player) = self.players.get_mut(&id) {
            player.position = position.clone();
            player.rotation = rotation;
        }
        let cx = (position.x / 32.0).floor() as i32;
        let cy = (position.y / 32.0).floor() as i32;
        let cz = (position.z / 32.0).floor() as i32;
        (cx, cy, cz)
    }

    pub fn set_player_chunks(&mut self, id: u64, chunks: HashSet<(i32, i32, i32)>) {
        self.player_chunks.insert(id, chunks);
    }

    pub fn all_required_chunks(&self) -> HashSet<(i32, i32, i32)> {
        self.player_chunks.values().flat_map(|chunks| chunks.iter()).cloned().collect()
    }
}

impl Default for PlayerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
