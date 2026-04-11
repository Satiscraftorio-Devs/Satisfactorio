use sha2::{Digest, Sha256};

pub const SALT: &[u8] = b"Satisfactorio_v1_LE48TRUC48DE48FOU";

pub fn generate_server_id() -> [u8; 16] {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes).expect("Failed to generate random bytes");
    bytes
}

pub fn server_id_to_hex(id: &[u8]) -> String {
    id.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn compute_shared_secret(server_id: &[u8], token: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(server_id);
    hasher.update(SALT);
    hasher.update(token);
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

pub fn xor_crypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter().enumerate().map(|(i, &b)| b ^ key[i % key.len()]).collect()
}
