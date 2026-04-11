mod network;

use shared::network::crypto::generate_server_id;
use shared::network::messages;
use shared::*;

use anyhow::Result;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::net::*;

static NEXT_PLAYER_ID: AtomicU64 = AtomicU64::new(1);

async fn handle_client(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let player_id = NEXT_PLAYER_ID.fetch_add(1, Ordering::SeqCst);
    let server_id = generate_server_id();
    log!("Nouveau joueur avec ID: {} (Server ID: {:02x?})", player_id, server_id);

    let mut conn = network::ServerConnection::new(player_id, server_id);

    conn.send_server_id(&mut stream).await?;

    let packet = match conn.receive_packet(&mut stream).await {
        Ok(p) => p,
        Err(e) => {
            log_err!("Erreur reception: {}", e);
            return Ok(());
        }
    };

    conn.handle_packet(packet);

    let ack = messages::create_handshake_ack(player_id, 0);
    if let Err(e) = conn.send_packet(&mut stream, ack).await {
        log_err!("Erreur envoi handshake ack: {}", e);
        return Ok(());
    }

    loop {
        match conn.receive_packet(&mut stream).await {
            Ok(packet) => conn.handle_packet(packet),
            Err(e) => {
                log_err!("Erreur réception paquet: {}", e);
                break;
            }
        }
    }

    log!("Joueur {} déconnecté", player_id);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:5000").await?;

    log!("Serveur démarre sur 127.0.0.1:5000");

    loop {
        let (stream, addr) = listener.accept().await?;
        log!("Connexion de {}", addr);

        tokio::spawn(async move {
            if let Err(e) = handle_client(stream).await {
                log_err!("Erreur handling client: {}", e);
            }
        });
    }
}
