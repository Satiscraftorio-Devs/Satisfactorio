use quinn::{RecvStream, SendStream};
use shared::network::crypto::compute_shared_secret;
use shared::network::messages::{ContenuPaquet, Paquet, MAX_PAQUET_SIZE};
use shared::network::network_protocol::{create_codec, EncryptedCodec};
use tokio::io::AsyncReadExt;

pub struct ServerConnection {
    codec: EncryptedCodec,
    player_id: u64,
    server_id: [u8; 16],
}

impl ServerConnection {
    pub fn new(player_id: u64, server_id: [u8; 16]) -> Self {
        let codec = create_codec(compute_shared_secret(&server_id, b"server"));

        Self {
            codec,
            player_id,
            server_id,
        }
    }

    pub async fn send_packet(&mut self, send: &mut SendStream, packet: Paquet) -> Result<(), String> {
        let data = self.codec.encode(&packet);
        send.write_all(&data).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn send_server_id(&mut self, send: &mut SendStream) -> Result<(), String> {
        send.write_all(&self.server_id).await.map_err(|e| e.to_string())
    }

    pub async fn receive_packet(&mut self, recv: &mut RecvStream) -> Result<Paquet, String> {
        let mut len_buf = [0u8; 4];
        recv.read_exact(&mut len_buf).await.map_err(|e| e.to_string())?;
        let len = u32::from_be_bytes(len_buf) as usize;

        if len > MAX_PAQUET_SIZE {
            return Err(format!("Paquet trop grand: {}", len));
        }

        let mut data = vec![0u8; len];
        recv.read_exact(&mut data).await.map_err(|e| e.to_string())?;

        let packet = self.codec.decode(&data)?;
        Ok(packet)
    }

    pub fn handle_packet(&self, packet: Paquet) {
        match packet.contenu {
            ContenuPaquet::DonneesConnexion { version, username } => {
                println!(
                    "Joueur {} (ID: {}) se connecte avec la version {}",
                    username, self.player_id, version
                );
            }
            ContenuPaquet::MessageChat { sender_id, content } => {
                println!("Message de {}: {}", sender_id, content);
            }
            ContenuPaquet::Deplacement {
                player_id,
                position,
                rotation: _,
            } => {
                println!("Joueur {} bouge vers ({}, {}, {})", player_id, position.x, position.y, position.z);
            }
            _ => {}
        }
    }
}
