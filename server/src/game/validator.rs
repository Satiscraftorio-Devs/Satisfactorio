//! Validateur de chunks côté serveur.
//!
//! Ce module valide les chunks générés par les clients pour détecter toute tricherie.
//! Le serveur génère le même chunk avec la même seed et compare les checksums.

use shared::world::data::chunk::Chunk;
use shared::*;
use std::collections::HashMap;

/// Nombre maximum de tentatives de validation autorisé avant de kicker le joueur.
const MAX_VALIDATION_ATTEMPT: u8 = 3;

/// Résultat d'une validation de chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationResult {
    /// Le chunk est valide (checksum identique)
    Valid,
    /// Le chunk est invalide.
    /// Si `should_kick` est true, le joueur doit être éjecté.
    Invalid { should_kick: bool },
}

/// Validateur de chunks.
///
/// Ce struct suit les tentatives de validation pour chaque chunk et kick le joueur
/// après trop d'échecs consécutifs. Cela détecte les clients qui trichent en modifiant
/// les données du chunk.
///
/// # Mécanisme de validation
///
/// 1. Le client envoie un chunk généré localement avec un checksum
/// 2. Le serveur génère le même chunk avec la même seed
/// 3. Le serveur calcule son propre checksum
/// 4. Si les checksums correspondent → Valide
/// 5. Si les checksums différent → Invalide (compte les tentatives)
///
/// # Détection de triche
///
/// Si un client envoie un chunk qui ne correspond pas à ce que le serveur génère,
/// cela peut signifier :
/// - Le client a une seed différente
/// - Le client a modifié l'algorithme de génération
/// - Le client envoie des données aléatoires
pub struct ChunkValidator {
    /// Map des tentatives échouées par coordonnée de chunk.
    /// Clé: (x, y, z) du chunk
    /// Valeur: nombre de tentatives
    failed_attempts: HashMap<(i32, i32, i32), u8>,
}

impl ChunkValidator {
    /// Crée un nouveau validateur.
    pub fn new() -> Self {
        Self {
            failed_attempts: HashMap::new(),
        }
    }

    /// Valide un chunk envoyé par le client.
    ///
    /// # Arguments
    ///
    /// * `x, y, z` - Coordonnées du chunk
    /// * `checksum` - Checksum envoyé par le client
    /// * `seed` - Seed du serveur pour générer le chunk de référence
    ///
    /// # Returns
    ///
    /// * `ValidationResult::Valid` si les checksums correspondent
    /// * `ValidationResult::Invalid { should_kick }` sinon
    ///
    /// # Effets de bord
    ///
    /// Met à jour le compteur de tentatives pour ce chunk.
    /// Si le nombre de tentatives max est dépassé, `should_kick` est true.
    pub fn validate(&mut self, x: i32, y: i32, z: i32, checksum: Vec<u8>, seed: u32) -> ValidationResult {
        // Génère le chunk de référence avec la seed du serveur
        let chunk = Chunk::generate(x, y, z, seed);

        // Calcule le checksum côté serveur
        let server_checksum = chunk.compute_checksum();

        // Compare les checksums
        let valide = checksum == server_checksum;

        if valide {
            // Succès : reset le compteur pour ce chunk
            self.failed_attempts.remove(&(x, y, z));
            // log_server!("Chunk ({}, {}, {}) Valide !", x, y, z);
            ValidationResult::Valid
        } else {
            // Échec : incrémente le compteur
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

            // Vérifie si on doit kicker le joueur
            let should_kick = *attempt >= MAX_VALIDATION_ATTEMPT;
            if should_kick {
                // Reset après kick
                self.failed_attempts.remove(&key);
            }

            ValidationResult::Invalid { should_kick }
        }
    }

    /// Réinitialise tous les compteurs de tentatives.
    ///
    /// Utile lors de la reconnexion d'un joueur ou pour tester.
    pub fn clear(&mut self) {
        self.failed_attempts.clear();
    }
}

impl Default for ChunkValidator {
    fn default() -> Self {
        Self::new()
    }
}
