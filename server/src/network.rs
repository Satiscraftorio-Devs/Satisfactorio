use shared::network::crypto::compute_shared_secret;
use shared::network::messages::{self, ContenuPaquet, Paquet, MAX_PAQUET_SIZE};
use shared::network::network_protocol::{create_codec, EncryptedCodec};
use shared::world::data::chunk::{Chunk, CHUNK_BLOCK_NUMBER};
use shared::*;
use std::sync::atomic::AtomicU32;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const MAX_VALIDATION_ATTEMPT: u8 = 3;

pub struct ServerConnection {
    codec: EncryptedCodec,
    player_id: u64,
    server_id: [u8; 16],
    failed_validation: std::collections::HashMap<(i32, i32, i32), u8>,
}

impl ServerConnection {
    pub fn new(player_id: u64, server_id: [u8; 16]) -> Self {
        let codec = create_codec(compute_shared_secret(&server_id, b"server"));

        Self {
            codec,
            player_id,
            server_id,
            failed_validation: std::collections::HashMap::new(),
        }
    }

    pub async fn send_packet(&mut self, stream: &mut tokio::net::TcpStream, packet: Paquet) -> Result<(), String> {
        let data = self.codec.encode(&packet);
        static SERVER_SEED: AtomicU32 = AtomicU32::new(0);
        let len = data.len() as u32;
        stream.write_all(&len.to_be_bytes()).await.map_err(|e| e.to_string())?;
        stream.write_all(&data).await.map_err(|e| e.to_string())?;
        stream.flush().await.map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn send_server_id(&mut self, stream: &mut tokio::net::TcpStream) -> Result<(), String> {
        stream.write_all(&self.server_id).await.map_err(|e| e.to_string())?;
        stream.flush().await.map_err(|e| e.to_string())
    }

    pub async fn receive_packet(&mut self, stream: &mut tokio::net::TcpStream) -> Result<Paquet, String> {
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await.map_err(|e| e.to_string())?;
        let len = u32::from_be_bytes(len_buf) as usize;

        if len > MAX_PAQUET_SIZE {
            return Err(format!("Paquet trop grand: {}", len));
        }

        let mut data = vec![0u8; len];
        stream.read_exact(&mut data).await.map_err(|e| e.to_string())?;

        let packet = self.codec.decode(&data)?;
        Ok(packet)
    }

    pub fn handle_packet(&mut self, packet: Paquet) -> Option<Paquet> {
        match packet.contenu {
            ContenuPaquet::DonneesConnexion { version, ref username } => {
                log_server!(
                    "Joueur {} (ID: {}) se connecte avec la version {}",
                    username,
                    self.player_id,
                    version
                );
                return Some(packet);
            }
            ContenuPaquet::Deplacement {
                player_id,
                ref position,
                rotation: _,
            } => {
                // log_server!("Joueur {} bouge vers ({}, {}, {})", player_id, position.x, position.y, position.z);
                return Some(packet);
            }
            ContenuPaquet::ChunkValidationRequest { x, y, z, checksum } => {
                let seed = crate::get_server_seed();
                let chunk = Chunk::generate(x, y, z, seed);
                let server_checksum = chunk.compute_checksum();
                let valide = checksum == server_checksum;

                if valide {
                    self.failed_validation.remove(&(x, y, z));
                    log_server!("Chunk ({}, {}, {}) Validé !", x, y, z);
                } else {
                    let key = (x, y, z);
                    let attempt = self.failed_validation.entry(key).or_insert(0);
                    *attempt += 1;
                    log_err_server!(
                        "Chunk ({}, {}, {}) invalide ! Tentative {}/{}",
                        x,
                        y,
                        z,
                        *attempt,
                        MAX_VALIDATION_ATTEMPT
                    );
                    if *attempt >= MAX_VALIDATION_ATTEMPT {
                        log_server!("Joueur {} explulsé, trop de valisation échouées", self.player_id);
                        return None;
                    }
                }
                return Some(messages::new_chunk_validation_response(x, y, z, valide, !valide));
            }

            _ => None,
        }
    }
}
