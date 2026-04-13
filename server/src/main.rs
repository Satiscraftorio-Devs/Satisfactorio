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

static NEXT_PLAYER_ID: AtomicU64 = AtomicU64::new(1);

async fn handle_client(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let player_id = NEXT_PLAYER_ID.fetch_add(1, Ordering::SeqCst);
    let server_id = generate_server_id();
    log_server!("Nouveau joueur avec ID: {} (Server ID: {:02x?})", player_id, server_id);

    let conn = ServerConnection::new(player_id, server_id);

    conn.send_server_id(&mut stream).await?;

    let packet = match conn.receive_packet(&mut stream).await {
        Ok(p) => p,
        Err(e) => {
            log_err_server!("Erreur reception: {}", e);
            return Ok(());
        }
    };

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

    let mut handler = PacketHandler::new();
    handler.handle_packet(packet);

    loop {
        match conn.receive_packet(&mut stream).await {
            Ok(packet) => {
                if let Some(response) = handler.handle_packet(packet) {
                    conn.send_packet(&mut stream, &response).await?;
                } else {
                    log_server!("Le joueur {} a ete ejecte", player_id);
                    break;
                }
            }
            Err(e) => {
                log_err_server!("Erreur reception paquet: {}", e);
                break;
            }
        }
    }

    log_server!("Joueur {} deconnecte", player_id);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    init_server_seed();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:5000").await?;

    log_server!("Serveur demarre sur 127.0.0.1:5000");

    loop {
        let (stream, addr) = listener.accept().await?;
        log_server!("Connexion de {}", addr);

        tokio::spawn(async move {
            if let Err(e) = handle_client(stream).await {
                log_err_server!("Erreur handling client: {}", e);
            }
        });
    }
}
