use shared::log_client;
use shared::network::crypto::compute_shared_secret;
use shared::network::messages::{ContenuPaquet, Paquet, Position, Rotation, CURRENT_VERSION};
use shared::network::network_protocol::{create_codec, EncryptedCodec};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;

pub struct NetworkClient {
    runtime: Option<Runtime>,
    stream: Option<TcpStream>,
    codec: Arc<EncryptedCodec>,
    player_id: Option<u64>,
    connected: bool,
}

impl NetworkClient {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            runtime: None,
            stream: None,
            codec: Arc::new(create_codec([0u8; 32])),
            player_id: None,

            connected: false,
        })
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn connect(&mut self, server_addr: &str) -> Result<(), String> {
        let runtime = Runtime::new().map_err(|e| e.to_string())?;
        let stream =
            runtime.block_on(async { Ok::<TcpStream, String>(TcpStream::connect(server_addr).await.map_err(|e| e.to_string())?) })?;

        self.runtime = Some(runtime);
        self.stream = Some(stream);
        Ok(())
    }

    pub fn perform_handshake(&mut self, username: &str) -> Result<(u64, u32), String> {
        let runtime = self.runtime.as_ref().ok_or("No runtime")?;

        let (player_id, server_seed, new_codec) = runtime.block_on(async {
            let mut stream = self.stream.as_mut().ok_or("Not connected")?;

            let mut server_id_buf = [0u8; 16];
            stream.read_exact(&mut server_id_buf).await.map_err(|e| e.to_string())?;

            let new_codec = Arc::new(create_codec(compute_shared_secret(&server_id_buf, b"server")));

            let packet = Paquet::new(
                0,
                shared::network::messages::TypePaquet::Handshake,
                ContenuPaquet::DonneesConnexion {
                    version: CURRENT_VERSION,
                    username: username.to_string(),
                },
            );

            let data = new_codec.encode(&packet);
            let len = data.len() as u32;
            stream.write_all(&len.to_be_bytes()).await.map_err(|e| e.to_string())?;
            stream.write_all(&data).await.map_err(|e| e.to_string())?;
            stream.flush().await.map_err(|e| e.to_string())?;

            let mut len_buf = [0u8; 4];
            stream.read_exact(&mut len_buf).await.map_err(|e| e.to_string())?;
            let len = u32::from_be_bytes(len_buf) as usize;
            let mut data = vec![0u8; len];
            stream.read_exact(&mut data).await.map_err(|e| e.to_string())?;

            let packet = new_codec.decode(&data).map_err(|e| e.to_string())?;
            let player_id = match packet.contenu {
                ContenuPaquet::Confirmation { player_id, .. } => player_id,
                _ => return Err("Unexpected packet".to_string()),
            };

            let seed_packet = new_codec.receive_packet(&mut stream).await.map_err(|e| e.to_string())?;
            let server_seed = match seed_packet.contenu {
                ContenuPaquet::ServerSeed { seed } => seed,
                _ => return Err("Expected server seed".to_string()),
            };
            log_client!("Server seed recue: {}", server_seed);

            Ok((player_id, server_seed as u32, new_codec))
        })?;

        self.codec = new_codec;
        self.player_id = Some(player_id);
        self.connected = true;
        Ok((player_id, server_seed))
    }

    pub fn send_position(&mut self, x: f32, y: f32, z: f32, rx: f32, ry: f32) -> Result<(), String> {
        if !self.connected {
            return Ok(());
        }

        let stream = self.stream.as_mut().ok_or("Not connected")?;
        let player_id = self.player_id.ok_or("Not authenticated")?;
        let codec = self.codec.clone();

        self.runtime.as_ref().ok_or("No runtime")?.block_on(async {
            let packet = Paquet::new(
                player_id,
                shared::network::messages::TypePaquet::PlayerUpdate,
                ContenuPaquet::Deplacement {
                    player_id,
                    position: Position { x, y, z },
                    rotation: Rotation { x: rx, y: ry },
                },
            );

            let data = codec.encode(&packet);
            let len = data.len() as u32;
            stream.write_all(&len.to_be_bytes()).await.map_err(|e| e.to_string())?;
            stream.write_all(&data).await.map_err(|e| e.to_string())?;
            stream.flush().await.map_err(|e| e.to_string())?;
            Ok(())
        })
    }

    pub fn send_packet(&mut self, packet: Paquet) -> Result<(), String> {
        if !self.connected {
            return Ok(());
        }

        let stream = self.stream.as_mut().ok_or("Not connected")?;
        let codec = self.codec.clone();

        self.runtime
            .as_ref()
            .ok_or("No runtime")?
            .block_on(async { codec.send_packet(&mut *stream, &packet).await })
    }
}
