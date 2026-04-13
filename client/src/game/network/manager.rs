use crate::engine::network::ClientConnection;
use crate::game::network::protocol::GameProtocol;
use shared::{log_client, network::messages::Paquet};
use std::time::{Duration, Instant};

const POSITION_UPDATE_INTERVAL: Duration = Duration::from_millis(50);

pub struct NetworkManager {
    connection: ClientConnection,
    protocol: Option<GameProtocol>,
    last_send: Instant,
    server_seed: Option<u64>,
}

impl NetworkManager {
    pub fn new() -> Self {
        Self {
            connection: ClientConnection::new().expect("Failed to create client connection"),
            protocol: None,
            last_send: Instant::now(),
            server_seed: None,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connection.is_connected()
    }

    pub fn player_id(&self) -> Option<u64> {
        self.connection.player_id()
    }

    pub fn connect(&mut self, server_addr: &str) {
        println!("NetworkManager: tentative de connexion...");
        if let Err(e) = self.connection.connect(server_addr) {
            println!("NetworkManager: erreur connexion: {}", e);
        }
    }

    pub fn perform_handshake(&mut self, username: &str) -> Result<u64, String> {
        println!("NetworkManager: handshake...");
        match self.connection.perform_handshake(username) {
            Ok((id, seed)) => {
                self.protocol = Some(GameProtocol::new(id));
                self.server_seed = Some(seed as u64);
                println!("NetworkManager: connecte!");
                Ok(id)
            }
            Err(e) => {
                println!("NetworkManager: erreur handshake: {}", e);
                Err(e)
            }
        }
    }

    pub fn get_server_seed(&self) -> Option<u32> {
        self.server_seed.map(|s| s as u32)
    }

    pub fn send_position(&mut self, x: f32, y: f32, z: f32, rx: f32, ry: f32) -> Result<(), String> {
        let now = Instant::now();
        if now.duration_since(self.last_send) >= POSITION_UPDATE_INTERVAL {
            if let Some(protocol) = &self.protocol {
                let packet = protocol.create_position_update(x, y, z, rx, ry);
                self.connection.send_packet(packet)?;
                self.last_send = now;
            }
        }
        Ok(())
    }

    pub fn send_chunk_validation(&mut self, x: i32, y: i32, z: i32, checksum: Vec<u8>) -> Result<(), String> {
        if let Some(protocol) = &self.protocol {
            let packet = protocol.create_chunk_validation_request(x, y, z, checksum);
            log_client!("Envoi chunk validation ({}, {}, {})", x, y, z);
            self.connection.send_packet(packet)
        } else {
            Ok(())
        }
    }

    pub fn receive_packet(&mut self) -> Result<Paquet, String> {
        self.connection.receive_packet()
    }
}
