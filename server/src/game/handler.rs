//! Gestionnaire de paquets côté serveur.
//!
//! Ce module traite les paquets reçus des clients et génère les réponses appropriées.
//! Il fait le lien entre la couche réseau (`ServerConnection`) et la logique de jeu.

use crate::game::validator::{ChunkValidator, ValidationResult};
use crate::world::get_server_seed;
use shared::network::messages::{self, ContenuPaquet, Paquet};
use shared::*;

/// Gestionnaire de paquets pour le serveur.
///
/// Ce struct est responsable du traitement des paquets reçus des clients.
/// Il délègue la validation des chunks à `ChunkValidator` et retourne
/// les réponses appropriées.
///
/// # Flux de traitement
///
/// ```text
/// Paquet reçu → handle_packet()
///     │
///     ├─ DonneesConnexion → Log connexion, retourne le paquet
///     ├─ Deplacement      → Log déplacement, retourne le paquet  
///     ├─ ChunkValidationRequest → Délègue à ChunkValidator
///     │                           → Retourne ChunkValidationResponse
///     └─Autre              → Ignore (retourne None)
/// ```
pub struct PacketHandler {
    /// Validateur de chunks (suit les tentatives de validation)
    validator: ChunkValidator,
}

impl PacketHandler {
    /// Crée un nouveau gestionnaire de paquets.
    pub fn new() -> Self {
        Self {
            validator: ChunkValidator::new(),
        }
    }

    /// Traite un paquet reçu et retourne une réponse éventuel.
    ///
    /// # Arguments
    ///
    /// * `packet` - Le paquet reçu du client
    ///
    /// # Returns
    ///
    /// * `Some(Paquet)` - Un paquet réponse à envoyer au client
    /// * `None` - Le joueur doit être déconnecté (trop d'échecs)
    pub fn handle_packet(&mut self, packet: Paquet) -> Option<Paquet> {
        // Analyse le type de contenu du paquet
        match packet.contenu {
            // === Connexion ===
            // Paquet de connexion initial (Handshake)
            ContenuPaquet::DonneesConnexion { version, ref username } => {
                log_server!("Joueur {} se connecte avec la version {}", username, version);
                // Renvoie le paquet tel quel (le codec l'enverra au client)
                Some(packet)
            }

            // === Déplacement ===
            // Mise à jour de position du joueur
            ContenuPaquet::Deplacement {
                player_id: _,
                position: _,
                rotation: _,
            } => Some(packet), // TODO: Stocker la position pour le multiplayer

            // === Validation de chunk ===
            // Le client envoie un chunk généré localement pour validation
            ContenuPaquet::ChunkValidationRequest { x, y, z, checksum } => {
                log_server!("Reception chunk validation ({}, {}, {})", x, y, z);

                // Récupère la seed du serveur pour générer le chunk de référence
                let seed = get_server_seed();

                // Valide le chunk avec le validateur
                let result = self.validator.validate(x, y, z, checksum, seed);

                match result {
                    // Chunk valide : génère une réponse positive
                    ValidationResult::Valid => Some(messages::new_chunk_validation_response(x, y, z, true, false)),
                    // Chunk invalide
                    ValidationResult::Invalid { should_kick } => {
                        // Génère la réponse correspondante
                        let response = messages::new_chunk_validation_response(x, y, z, false, should_kick);

                        // Si trop de tentatives échouées,Kick le joueur
                        if should_kick {
                            log_server!("Trop de validations echouees, joueur expulse");
                            return None;
                        }
                        Some(response)
                    }
                }
            }

            // === Type inconnu ===
            // Ignore les paquets non gérés
            _ => None,
        }
    }
}

impl Default for PacketHandler {
    fn default() -> Self {
        Self::new()
    }
}
