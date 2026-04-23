use crate::game::validator::{ChunkValidator, ValidationResult};
use crate::state::GAME_STATE;
use shared::network::messages::{self, ContenuPaquet, Paquet};
use shared::*;

pub struct PacketHandler {
    validator: ChunkValidator,
}

impl PacketHandler {
    pub fn new() -> Self {
        Self {
            validator: ChunkValidator::new(),
        }
    }

    pub fn handle_packet(&mut self, packet: Paquet) -> Option<Paquet> {
        match &packet.contenu {
            ContenuPaquet::DonneesConnexion { version, username } => {
                log_server!("Joueur {} se connecte avec la version {}", username, version);
                Some(packet)
            }

            ContenuPaquet::Deplacement {
                player_id,
                position,
                rotation,
            } => {
                GAME_STATE.update_player_position(*player_id, position.clone(), rotation.clone());
                Some(packet)
            }

            ContenuPaquet::ChunkValidationBatchRequest { chunks } => {
                log_server!("Reception ChunkValidationBatchRequest avec {} chunks", chunks.len());
                let seed = GAME_STATE.get_seed();
                let chunks = chunks.clone();
                let results = self.validator.validate_batch(chunks, seed, &GAME_STATE);
                log_server!("Envoi ChunkValidationBatchResponse avec {} résultats", results.len());
                Some(messages::new_chunk_validation_batch_response(results))
            }

            _ => None,
        }
    }
}

impl Default for PacketHandler {
    fn default() -> Self {
        Self::new()
    }
}
