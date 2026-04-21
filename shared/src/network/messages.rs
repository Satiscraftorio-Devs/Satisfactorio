use serde::{Deserialize, Serialize};

pub const CURRENT_VERSION: u8 = 1;
pub const MAX_PAQUET_SIZE: usize = 65536;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypePaquet {
    Handshake,
    HandshakeAck,
    ChunkValidationRequest,
    ChunkValidationResponse,
    ChunkValidationBatchRequest,
    ChunkValidationBatchResponse,
    ServerSeed,
    PlayerUpdate,
    WorldData,
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
    DonneesMonde {
        chunks: Vec<ChunkData>,
    },
    ChunkValidationRequest {
        x: i32,
        y: i32,
        z: i32,
        checksum: Vec<u8>,
    },
    ChunkValidationResponse {
        x: i32,
        y: i32,
        z: i32,
        valide: bool,
        regneration: bool,
    },
    ChunkValidationBatchRequest {
        chunks: Vec<BatchChunkChecksum>,
    },
    ChunkValidationBatchResponse {
        results: Vec<BatchValidationResult>,
    },
    ServerSeed {
        seed: u32,
    },
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BatchChunkChecksum {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub checksum: [u8; 2],
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BatchValidationResult {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub valide: bool,
    pub regneration: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

pub fn new_chunk_validation_request(x: i32, y: i32, z: i32, checksum: Vec<u8>) -> Paquet {
    Paquet {
        id: 0,
        type_paquet: TypePaquet::ChunkValidationRequest,
        contenu: ContenuPaquet::ChunkValidationRequest { x, y, z, checksum },
    }
}

pub fn new_chunk_validation_response(x: i32, y: i32, z: i32, valide: bool, regneration: bool) -> Paquet {
    Paquet {
        id: 0,
        type_paquet: TypePaquet::ChunkValidationResponse,
        contenu: ContenuPaquet::ChunkValidationResponse {
            x,
            y,
            z,
            valide,
            regneration,
        },
    }
}

pub fn new_chunk_validation_batch_request(chunks: Vec<BatchChunkChecksum>) -> Paquet {
    Paquet {
        id: 0,
        type_paquet: TypePaquet::ChunkValidationBatchRequest,
        contenu: ContenuPaquet::ChunkValidationBatchRequest { chunks },
    }
}

pub fn new_chunk_validation_batch_response(results: Vec<BatchValidationResult>) -> Paquet {
    Paquet {
        id: 0,
        type_paquet: TypePaquet::ChunkValidationBatchResponse,
        contenu: ContenuPaquet::ChunkValidationBatchResponse { results },
    }
}

pub fn new_server_seed_paquet(seed: u32) -> Paquet {
    Paquet::new(0, TypePaquet::ServerSeed, ContenuPaquet::ServerSeed { seed })
}
