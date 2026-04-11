use crate::network::crypto::{compute_shared_secret, generate_server_id, server_id_to_hex, xor_crypt};
use crate::network::messages::{Paquet, MAX_PAQUET_SIZE};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Cipher {
    key: [u8; 32],
}

impl Cipher {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn from_shared_secret(shared_secret: [u8; 32]) -> Self {
        Self { key: shared_secret }
    }

    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        xor_crypt(data, &self.key)
    }

    pub fn decrypt(&self, data: &[u8]) -> Vec<u8> {
        xor_crypt(data, &self.key)
    }
}

pub struct EncryptedCodec {
    cipher: Arc<Cipher>,
}

impl EncryptedCodec {
    pub fn new(cipher: Arc<Cipher>) -> Self {
        Self { cipher }
    }

    pub fn encode(&self, packet: &Paquet) -> Vec<u8> {
        let serialized = packet.serialize();
        let encrypted = self.cipher.encrypt(&serialized);

        let len = encrypted.len() as u32;
        let mut result = Vec::with_capacity(4 + encrypted.len());
        result.extend_from_slice(&len.to_be_bytes());
        result.extend_from_slice(&encrypted);
        result
    }

    pub fn decode(&self, data: &[u8]) -> Result<Paquet, String> {
        if data.len() < 4 {
            return Err("Data too short for length prefix".to_string());
        }

        let len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;

        if len > MAX_PAQUET_SIZE {
            return Err(format!("Packet too large: {}", len));
        }

        if data.len() < 4 + len {
            return Err("Data too short for packet".to_string());
        }

        let encrypted = &data[4..4 + len];
        let decrypted = self.cipher.decrypt(encrypted);

        Paquet::deserialize(&decrypted).map_err(|e| e.to_string())
    }
}

pub fn perform_client_handshake(server_id_bytes: &[u8], client_token: &[u8]) -> [u8; 32] {
    compute_shared_secret(server_id_bytes, client_token)
}

pub fn perform_server_handshake(server_id_bytes: &[u8]) -> [u8; 32] {
    compute_shared_secret(server_id_bytes, b"server")
}

pub fn create_server_id() -> ([u8; 16], String) {
    let id = generate_server_id();
    let hex = server_id_to_hex(&id);
    (id, hex)
}

pub fn create_cipher(shared_secret: [u8; 32]) -> Arc<Cipher> {
    Arc::new(Cipher::from_shared_secret(shared_secret))
}

pub fn create_codec(shared_secret: [u8; 32]) -> EncryptedCodec {
    let cipher = create_cipher(shared_secret);
    EncryptedCodec::new(cipher)
}
