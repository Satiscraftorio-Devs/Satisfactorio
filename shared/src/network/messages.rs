use serde::{Deserialize, Serialize};

pub use crate::network::crypto::{compute_shared_secret, generate_server_id, server_id_to_hex, xor_crypt};

pub const CURRENT_VERSION: u8 = 1;
pub const MAX_PAQUET_SIZE: usize = 65536;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypePaquet {
    Handshake,
    HandshakeAck,
    PlayerUpdate,
    Chat,
    WorldData,
    Disconnect,
    Ping,
    Pong,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ContenuPaquet {
    DonneesConnexion {
        version: u8,
        username: String,
    },
    Confirmation {
        player_id: u64,
        server_time: u64,
    },
    Deplacement {
        player_id: u64,
        position: Position,
        rotation: Rotation,
    },
    MessageChat {
        sender_id: u64,
        content: String,
    },
    DonneesMonde {
        chunks: Vec<ChunkData>,
    },
    Deconnexion {
        reason: String,
    },
    Vide,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rotation {
    pub x: f32,
    pub y: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChunkData {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Paquet {
    pub id: u64,
    pub type_paquet: TypePaquet,
    pub contenu: ContenuPaquet,
}

impl Paquet {
    pub fn new(id: u64, type_paquet: TypePaquet, contenu: ContenuPaquet) -> Self {
        Self { id, type_paquet, contenu }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Failed to serialize packet")
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(data)
    }
}

pub fn create_handshake(username: String) -> Paquet {
    Paquet::new(
        0,
        TypePaquet::Handshake,
        ContenuPaquet::DonneesConnexion {
            version: CURRENT_VERSION,
            username,
        },
    )
}

pub fn create_handshake_ack(player_id: u64, server_time: u64) -> Paquet {
    Paquet::new(0, TypePaquet::HandshakeAck, ContenuPaquet::Confirmation { player_id, server_time })
}

pub fn create_player_update(player_id: u64, x: f32, y: f32, z: f32, rx: f32, ry: f32) -> Paquet {
    Paquet::new(
        0,
        TypePaquet::PlayerUpdate,
        ContenuPaquet::Deplacement {
            player_id,
            position: Position { x, y, z },
            rotation: Rotation { x: rx, y: ry },
        },
    )
}

pub fn create_chat(sender_id: u64, content: String) -> Paquet {
    Paquet::new(0, TypePaquet::Chat, ContenuPaquet::MessageChat { sender_id, content })
}

pub fn create_world_data(chunks: Vec<ChunkData>) -> Paquet {
    Paquet::new(0, TypePaquet::WorldData, ContenuPaquet::DonneesMonde { chunks })
}

pub fn create_disconnect(reason: String) -> Paquet {
    Paquet::new(0, TypePaquet::Disconnect, ContenuPaquet::Deconnexion { reason })
}

pub fn create_ping() -> Paquet {
    Paquet::new(0, TypePaquet::Ping, ContenuPaquet::Vide)
}

pub fn create_pong() -> Paquet {
    Paquet::new(0, TypePaquet::Pong, ContenuPaquet::Vide)
}
