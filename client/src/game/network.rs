use crate::engine::network::NetworkClient;
use std::time::{Duration, Instant};

const POSITION_UPDATE_INTERVAL: Duration = Duration::from_millis(50);

pub struct NetworkManager {
    client: NetworkClient,
    last_send: Instant,
    player_id: Option<u64>,
}

impl NetworkManager {
    pub fn new() -> Self {
        Self {
            client: NetworkClient::new().expect("Failed to create network client"),
            last_send: Instant::now(),
            player_id: None,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_connected()
    }

    pub fn get_player_id(&self) -> Option<u64> {
        self.player_id
    }

    pub fn connect(&mut self, server_addr: &str) {
        println!("NetworkManager: tentative de connexion...");
        if let Err(e) = self.client.connect(server_addr) {
            println!("NetworkManager: erreur connexion: {}", e);
        }
    }

    pub fn perform_handshake(&mut self, username: &str) -> Result<u64, String> {
        println!("NetworkManager: handshake...");
        match self.client.perform_handshake(username) {
            Ok(id) => {
                self.player_id = Some(id);
                println!("NetworkManager: connecte!");
                Ok(id)
            }
            Err(e) => {
                println!("NetworkManager: erreur handshake: {}", e);
                Err(e)
            }
        }
    }

    pub fn send_position(&mut self, x: f32, y: f32, z: f32, rx: f32, ry: f32) -> Result<(), String> {
        let now = Instant::now();
        if now.duration_since(self.last_send) >= POSITION_UPDATE_INTERVAL {
            self.client.send_position(x, y, z, rx, ry)?;
            self.last_send = now;
        }
        Ok(())
    }
}
