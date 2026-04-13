use shared::world::data::chunk::Chunk;
use shared::*;
use std::collections::HashMap;

const MAX_VALIDATION_ATTEMPT: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationResult {
    Valid,
    Invalid { should_kick: bool },
}

pub struct ChunkValidator {
    failed_attempts: HashMap<(i32, i32, i32), u8>,
}

impl ChunkValidator {
    pub fn new() -> Self {
        Self {
            failed_attempts: HashMap::new(),
        }
    }

    pub fn validate(&mut self, x: i32, y: i32, z: i32, checksum: Vec<u8>, seed: u32) -> ValidationResult {
        let chunk = Chunk::generate(x, y, z, seed);
        let server_checksum = chunk.compute_checksum();
        let valide = checksum == server_checksum;

        if valide {
            self.failed_attempts.remove(&(x, y, z));
            log_server!("Chunk ({}, {}, {}) Valide !", x, y, z);
            ValidationResult::Valid
        } else {
            let key = (x, y, z);
            let attempt = self.failed_attempts.entry(key).or_insert(0);
            *attempt += 1;
            log_err_server!(
                "Chunk ({}, {}, {}) invalide ! Tentative {}/{}",
                x,
                y,
                z,
                *attempt,
                MAX_VALIDATION_ATTEMPT
            );
            let should_kick = *attempt >= MAX_VALIDATION_ATTEMPT;
            if should_kick {
                self.failed_attempts.remove(&key);
            }
            ValidationResult::Invalid { should_kick }
        }
    }

    pub fn clear(&mut self) {
        self.failed_attempts.clear();
    }
}

impl Default for ChunkValidator {
    fn default() -> Self {
        Self::new()
    }
}
