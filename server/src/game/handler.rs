use crate::game::validator::{ChunkValidator, ValidationResult};
use crate::world::get_server_seed;
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
        match packet.contenu {
            ContenuPaquet::DonneesConnexion { version, ref username } => {
                log_server!("Joueur {} se connecte avec la version {}", username, version);
                Some(packet)
            }
            ContenuPaquet::Deplacement {
                player_id: _,
                position: _,
                rotation: _,
            } => Some(packet),
            ContenuPaquet::ChunkValidationRequest { x, y, z, checksum } => {
                log_server!("Reception chunk validation ({}, {}, {})", x, y, z);
                let seed = get_server_seed();
                let result = self.validator.validate(x, y, z, checksum, seed);

                match result {
                    ValidationResult::Valid => Some(messages::new_chunk_validation_response(x, y, z, true, false)),
                    ValidationResult::Invalid { should_kick } => {
                        let response = messages::new_chunk_validation_response(x, y, z, false, should_kick);
                        if should_kick {
                            log_server!("Trop de validations echouees, joueur expulse");
                            return None;
                        }
                        Some(response)
                    }
                }
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
