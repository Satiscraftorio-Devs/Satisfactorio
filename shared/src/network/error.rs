use std::fmt;

#[derive(Debug, Clone)]
pub enum NetworkError {
    Io(String),
    Codec(String),
    PacketTooLarge(usize),
    InvalidPacket(String),
    Disconnected,
    NotConnected,
    InvalidData(String),
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::Io(e) => write!(f, "IO error: {}", e),
            NetworkError::Codec(e) => write!(f, "Codec error: {}", e),
            NetworkError::PacketTooLarge(size) => write!(f, "Packet too large: {} bytes", size),
            NetworkError::InvalidPacket(e) => write!(f, "Invalid packet: {}", e),
            NetworkError::Disconnected => write!(f, "Connection closed"),
            NetworkError::NotConnected => write!(f, "Not connected"),
            NetworkError::InvalidData(e) => write!(f, "Invalid data: {}", e),
        }
    }
}

impl std::error::Error for NetworkError {}

impl From<std::io::Error> for NetworkError {
    fn from(err: std::io::Error) -> Self {
        NetworkError::Io(err.to_string())
    }
}

impl From<String> for NetworkError {
    fn from(err: String) -> Self {
        NetworkError::Codec(err)
    }
}

impl From<&str> for NetworkError {
    fn from(err: &str) -> Self {
        NetworkError::Codec(err.to_string())
    }
}
