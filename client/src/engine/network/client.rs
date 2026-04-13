use shared::network::crypto::compute_shared_secret;
use shared::network::messages::{ContenuPaquet, Paquet, CURRENT_VERSION};
use shared::network::network_protocol::{create_codec, EncryptedCodec};
use shared::network::traits::PacketCodec;
use shared::{log_client, log_err_client};
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

pub struct ClientConnection {
    runtime: Option<Runtime>,
    stream: Option<Arc<Mutex<TcpStream>>>,
    codec: Arc<EncryptedCodec>,
    player_id: Option<u64>,
    connected: bool,
}

impl ClientConnection {
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

    pub fn player_id(&self) -> Option<u64> {
        self.player_id
    }

    pub fn connect(&mut self, server_addr: &str) -> Result<(), String> {
        let runtime = Runtime::new().map_err(|e| e.to_string())?;
        let stream =
            runtime.block_on(async { Ok::<TcpStream, String>(TcpStream::connect(server_addr).await.map_err(|e| e.to_string())?) })?;

        self.runtime = Some(runtime);
        self.stream = Some(Arc::new(Mutex::new(stream)));
        Ok(())
    }

    pub fn perform_handshake(&mut self, username: &str) -> Result<(u64, u32), String> {
        let runtime = self.runtime.as_ref().ok_or("No runtime")?;

        let (player_id, server_seed, new_codec) = runtime.block_on(async {
            let mut stream = self.stream.as_mut().ok_or("Not connected")?.lock().await;

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

            new_codec.send_packet(&mut *stream, &packet).await.map_err(|e| e.to_string())?;

            let confirmation_packet = new_codec.receive_packet(&mut *stream).await.map_err(|e| e.to_string())?;
            let player_id = match confirmation_packet.contenu {
                ContenuPaquet::Confirmation { player_id, .. } => player_id,
                _ => return Err("Unexpected packet".to_string()),
            };

            let seed_packet = new_codec.receive_packet(&mut *stream).await.map_err(|e| e.to_string())?;
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

    pub fn send_packet(&mut self, packet: Paquet) -> Result<(), String> {
        if !self.connected {
            return Ok(());
        }

        let stream = self.stream.clone();
        let codec = self.codec.clone();

        if let Some(stream) = stream {
            let runtime = self.runtime.as_ref().ok_or("No runtime")?;
            runtime.spawn(async move {
                let guard = stream.lock().await;
                let mut stream = guard;
                if let Err(e) = codec.send_packet(&mut *stream, &packet).await {
                    log_err_client!("Erreur envoi packet: {}", e);
                }
            });
        }

        Ok(())
    }

    pub fn receive_packet(&mut self) -> Result<Paquet, String> {
        if !self.connected {
            return Err("Not connected".to_string());
        }

        let runtime = self.runtime.as_ref().ok_or("No runtime")?;
        let stream = self.stream.as_mut().ok_or("No stream")?;
        let codec = self.codec.clone();

        runtime.block_on(async {
            let mut stream = stream.lock().await;
            codec.receive_packet(&mut *stream).await.map_err(|e| e.to_string())
        })
    }
}
