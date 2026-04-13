use crate::network::error::NetworkError;
use crate::network::messages::Paquet;
use tokio::io::{AsyncRead, AsyncWrite};

pub trait PacketCodec: Clone + Send + Sync {
    async fn send_packet<S: AsyncRead + AsyncWrite + Unpin>(&self, stream: &mut S, packet: &Paquet) -> Result<(), NetworkError>;
    async fn receive_packet<S: AsyncRead + AsyncWrite + Unpin>(&self, stream: &mut S) -> Result<Paquet, NetworkError>;
}
