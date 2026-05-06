use shared::network::messages::PlayerTransformation;
use std::collections::HashMap;

use crate::common::utils::updatable::Updatable;

pub struct RemotePlayer {
    pub player_id: u64,
    pub position: Updatable<(f32, f32, f32)>,
    pub rotation: Updatable<(f32, f32)>,
    pub mesh_id: Option<u32>,
}

pub struct RemotePlayersManager {
    players: HashMap<u64, RemotePlayer>,
}

impl RemotePlayersManager {
    pub fn new() -> Self {
        Self { players: HashMap::new() }
    }

    pub fn update(&mut self, transforms: Vec<PlayerTransformation>, my_id: u64) {
        for t in transforms {
            if t.player_id != my_id {
                let entry = self.players.entry(t.player_id);
                let player = entry.or_insert_with(|| RemotePlayer {
                    player_id: t.player_id,
                    position: Updatable::new((0.0, 0.0, 0.0)),
                    rotation: Updatable::new((0.0, 0.0)),
                    mesh_id: None,
                });
                player.position.update((t.position.x, t.position.y, t.position.z));
                player.rotation.update((t.rotation.x, t.rotation.y));
            }
        }
    }

    pub fn get_all(&self) -> Vec<&RemotePlayer> {
        self.players.values().collect()
    }

    pub fn get_all_mut(&mut self) -> Vec<&mut RemotePlayer> {
        self.players.values_mut().collect()
    }
}
