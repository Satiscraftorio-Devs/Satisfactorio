use shared::network::messages::{ContenuPaquet, Paquet, Position, Rotation};

pub struct GameProtocol {
    player_id: u64,
}

impl GameProtocol {
    pub fn new(player_id: u64) -> Self {
        Self { player_id }
    }

    pub fn create_position_update(&self, x: f32, y: f32, z: f32, rx: f32, ry: f32) -> Paquet {
        Paquet::new(
            self.player_id,
            shared::network::messages::TypePaquet::PlayerUpdate,
            ContenuPaquet::Deplacement {
                player_id: self.player_id,
                position: Position { x, y, z },
                rotation: Rotation { x: rx, y: ry },
            },
        )
    }

    pub fn create_chunk_validation_request(&self, x: i32, y: i32, z: i32, checksum: Vec<u8>) -> Paquet {
        shared::network::messages::new_chunk_validation_request(x, y, z, checksum)
    }
}
