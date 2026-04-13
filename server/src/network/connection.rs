use shared::network::crypto::compute_shared_secret;
use shared::network::error::NetworkError;
use shared::network::messages::Paquet;
use shared::network::network_protocol::{create_codec, EncryptedCodec};
use shared::network::traits::PacketCodec;

pub struct ServerConnection {
    codec: EncryptedCodec,
    server_id: [u8; 16],
}

impl ServerConnection {
    pub fn new(player_id: u64, server_id: [u8; 16]) -> Self {
        let codec = create_codec(compute_shared_secret(&server_id, b"server"));
        let _ = player_id;
        Self { codec, server_id }
    }

    pub async fn send_packet<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin>(
        &self,
        stream: &mut S,
        packet: &Paquet,
    ) -> Result<(), NetworkError> {
        self.codec.send_packet(stream, packet).await
    }

    pub async fn receive_packet<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin>(
        &self,
        stream: &mut S,
    ) -> Result<Paquet, NetworkError> {
        self.codec.receive_packet(stream).await
    }

    pub async fn send_server_id<S: tokio::io::AsyncWrite + Unpin>(&self, stream: &mut S) -> Result<(), NetworkError> {
        use tokio::io::AsyncWriteExt;
        stream.write_all(&self.server_id).await?;
        stream.flush().await?;
        Ok(())
    }
}
