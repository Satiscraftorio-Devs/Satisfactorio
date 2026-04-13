//! Protocole applicatif côté client.
//!
//! Ce module définit le protocole de jeu, c'est-à-dire la création des paquets
//! métier utilisés par le client pour communiquer avec le serveur.
//!
//! Il fait la ponte entre la logique de jeu (positions, chunks) et le système
//! de paquets réseau.

use shared::network::messages::{ContenuPaquet, Paquet, Position, Rotation};

/// Protocol de jeu pour le client.
///
/// Cette structure est responsable de la création des paquets destinés au serveur.
/// Elle encapsule l'ID du joueur pour éviter de le répéter à chaque création.
///
/// # Types de paquets créés
///
/// - `PositionUpdate` : Envoi de la position/rotation du joueur
/// - `ChunkValidationRequest` : Demande de validation d'un chunk
pub struct GameProtocol {
    /// ID du joueur attribué par le serveur
    player_id: u64,
}

impl GameProtocol {
    /// Crée un nouveau protocole de jeu.
    ///
    /// # Arguments
    ///
    /// * `player_id` - ID du joueur attribué par le serveur
    pub fn new(player_id: u64) -> Self {
        Self { player_id }
    }

    /// Crée un paquet de mise à jour de position.
    ///
    /// Ce paquet est envoyé au serveur pour informer de la position
    /// et de la rotation du joueur.
    ///
    /// # Arguments
    ///
    /// * `x, y, z` - Position du joueur dans le monde
    /// * `rx, ry` - Rotation (yaw, pitch)
    ///
    /// # Returns
    ///
    /// Un `Paquet` prêt à être envoyé
    pub fn create_position_update(&self, x: f32, y: f32, z: f32, rx: f32, ry: f32) -> Paquet {
        Paquet::new(
            self.player_id,
            shared::network::messages::TypePaquet::PlayerUpdate,
            ContenuPaquet::Deplacement {
                player_id: self.player_id,
                position: Position { x, y, z },
                rotation: Rotation { x: rx, y: ry },
            },
        )
    }

    /// Crée un paquet de demande de validation de chunk.
    ///
    /// Ce paquet est envoyé au serveur pour vérifier qu'un chunk généré
    /// localement correspond au chunk qui serait généré par le serveur.
    /// Cela permet de détecter les tricheries.
    ///
    /// # Arguments
    ///
    /// * `x, y, z` - Coordonnées du chunk
    /// * `checksum` - Somme de contrôle du chunk généré localement
    ///
    /// # Returns
    ///
    /// Un `Paquet` prêt à être envoyé
    pub fn create_chunk_validation_request(&self, x: i32, y: i32, z: i32, checksum: Vec<u8>) -> Paquet {
        shared::network::messages::new_chunk_validation_request(x, y, z, checksum)
    }
}
