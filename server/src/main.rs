//! Serveur Satisfactorio - Point d'entrée.
//!
//! Ce fichier contient la boucle principale du serveur qui :
//! 1. Initialise la seed du monde
//! 2. Écoute les connexions TCP sur le port 5000
//! 3. Pour chaque client, lance une tâche async pour gérer la connexion
//!
//! ## Architecture du serveur
//!
//! ```text
//! main()
//!   ├── init_server_seed()           # Génère la seed du monde
//!   ├── TcpListener::bind()          # Écoute sur port 5000
//!   └── loop {
//!         └── tokio::spawn(handle_client)  # Pour chaque client
//!     }
//!
//! handle_client()
//!   ├── Envoie server_id (non chiffré)
//!   ├── Reçoit Handshake
//!   ├── Envoie HandshakeAck + ServerSeed
//!   └── Boucle de jeu :
//!         ├── receive_packet()
//!         ├── handler.handle_packet()
//!         └── send_packet(response)
//! ```

mod game;
mod network;
mod world;

use crate::game::PacketHandler;
use crate::network::ServerConnection;
use crate::world::*;
use shared::network::crypto::generate_server_id;
use shared::network::messages::{self, new_server_seed_paquet};
use shared::*;

use anyhow::Result;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::net::*;

/// Compteur atomique pour générer des IDs de joueurs uniques.
/// Commence à 1 pour éviter les conflits avec l'ID 0 (système).
static NEXT_PLAYER_ID: AtomicU64 = AtomicU64::new(1);

/// Gère la connexion d'un client.
///
/// Cette fonction est appelée pour chaque client qui se connecte.
/// Elle établit la connexion, effectue le handshake, puis entre dans
/// la boucle de traitement des paquets.
///
/// # Flux de connexion
///
/// 1. **Génération ID** : Attribue un ID unique au joueur
/// 2. **Envoi server_id** : Envoie l'ID du serveur (non chiffré)
/// 3. **Réception handshake** : Reçoit le paquet de connexion du client
/// 4. **Réponse handshake** : Envoie HandshakeAck + ServerSeed
/// 5. **Boucle jeu** : Traite les paquets dans une boucle infinie
///
/// # Gestion des erreurs
///
/// - Erreur de réception : Log et fin de la connexion
/// - Erreur d'envoi : Log et fin de la connexion
/// - Kick du joueur (handle_packet retourne None) : Fin de la connexion
async fn handle_client(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    // Étape 1: Attribuer un ID unique au joueur
    let player_id = NEXT_PLAYER_ID.fetch_add(1, Ordering::SeqCst);

    // Générer un server_id aléatoire pour le handshake
    let server_id = generate_server_id();
    log_server!("Nouveau joueur avec ID: {} (Server ID: {:02x?})", player_id, server_id);

    // Créer la connexion avec le codec approprié
    let conn = ServerConnection::new(player_id, server_id);

    // Étape 2: Envoyer le server_id (non chiffré)
    conn.send_server_id(&mut stream).await?;

    // Étape 3: Recevoir le packet de handshake du client
    let packet = match conn.receive_packet(&mut stream).await {
        Ok(p) => p,
        Err(e) => {
            log_err_server!("Erreur reception: {}", e);
            return Ok(());
        }
    };

    // Créer le gestionnaire de paquets pour ce client
    let mut handler = PacketHandler::new();

    // Traiter le premier paquet (généralement DonneesConnexion)
    handler.handle_packet(packet);

    // Étape 4: Envoyer le HandshakeAck et la seed du serveur
    let ack = messages::create_handshake_ack(player_id, 0);
    if let Err(e) = conn.send_packet(&mut stream, &ack).await {
        log_err_server!("Erreur envoi handshake ack: {}", e);
        return Ok(());
    }

    let seed_packet = new_server_seed_paquet(get_server_seed());
    if let Err(e) = conn.send_packet(&mut stream, &seed_packet).await {
        log_err_server!("Erreur envoi de la seed: {}", e);
        return Ok(());
    } else {
        log_server!("La Seed a ete envoyee au joueur {}", player_id);
    }

    // Étape 5: Boucle principale de traitement des paquets
    loop {
        match conn.receive_packet(&mut stream).await {
            Ok(packet) => {
                // Traiter le paquet et obtenir une réponse éventuelle
                if let Some(response) = handler.handle_packet(packet) {
                    // Envoyer la réponse au client
                    conn.send_packet(&mut stream, &response).await?;
                } else {
                    // Le validateur a décidé de kicker le joueur
                    log_server!("Le joueur {} a ete ejecte", player_id);
                    break;
                }
            }
            Err(e) => {
                // Erreur de réception (connexion fermée ou autre)
                log_err_server!("Erreur reception paquet: {}", e);
                break;
            }
        }
    }

    log_server!("Joueur {} deconnecte", player_id);
    Ok(())
}

/// Point d'entrée du serveur.
///
/// Initialise la seed du monde et lance le serveur TCP.
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Initialiser la seed du serveur pour la génération de chunks
    init_server_seed();

    // Créer le listener TCP sur le port 5000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:5000").await?;

    log_server!("Serveur demarre sur 127.0.0.1:5000");

    // Boucle d'acceptation des connexions
    loop {
        // Accepter une nouvelle connexion
        let (stream, addr) = listener.accept().await?;
        log_server!("Connexion de {}", addr);

        // Lancer une tâche pour gérer ce client
        // tokio::spawn permet de gérer plusieurs clients en parallèle
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream).await {
                log_err_server!("Erreur handling client: {}", e);
            }
        });
    }
}
