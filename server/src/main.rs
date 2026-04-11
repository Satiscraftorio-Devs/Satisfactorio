mod network;
use shared::*;

use anyhow::Result;
use quinn::{Endpoint, ServerConfig};
use rcgen::{generate_simple_self_signed, CertifiedKey};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use shared::network::crypto::generate_server_id;
use shared::network::messages;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_PLAYER_ID: AtomicU64 = AtomicU64::new(1);

fn configure_server() -> ServerConfig {
    log!("coucou");
    let CertifiedKey { cert, signing_key } = generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let cert_der = CertificateDer::from(cert.der().to_vec());
    let priv_key = PrivatePkcs8KeyDer::from(signing_key.serialize_der());

    ServerConfig::with_single_cert(vec![cert_der], priv_key.into()).unwrap()
}

async fn handle_client(mut send: quinn::SendStream, mut recv: quinn::RecvStream) {
    let player_id = NEXT_PLAYER_ID.fetch_add(1, Ordering::SeqCst);
    let server_id = generate_server_id();
    log!("Nouveau joueur avec ID: {} (Server ID: {:02x?})", player_id, server_id);

    let mut conn = network::ServerConnection::new(player_id, server_id);

    conn.send_server_id(&mut send).await.unwrap();

    let packet = match conn.receive_packet(&mut recv).await {
        Ok(p) => p,
        Err(e) => {
            log_err!("Erreur reception: {}", e);
            return;
        }
    };

    conn.handle_packet(packet);

    let ack = messages::create_handshake_ack(player_id, 0);
    if let Err(e) = conn.send_packet(&mut send, ack).await {
        log_err!("Erreur envoi handshake ack: {}", e);
        return;
    }

    let example_chunks = vec![messages::ChunkData {
        x: 0,
        y: 0,
        z: 0,
        data: vec![0u8; 16],
    }];
    let world_data = messages::create_world_data(example_chunks);
    if let Err(e) = conn.send_packet(&mut send, world_data).await {
        log_err!("Erreur envoi world data: {}", e);
        return;
    }

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    send.finish().ok();
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let addr = "127.0.0.1:5000".parse()?;
    let server_config = configure_server();
    let endpoint = Endpoint::server(server_config, addr)?;

    log!("Serveur demarre sur {}", addr);

    while let Some(connecting) = endpoint.accept().await {
        let connection = connecting.await?;
        log!("Connexion de {}", connection.remote_address());

        tokio::spawn(async move {
            if let Ok((send, recv)) = connection.accept_bi().await {
                handle_client(send, recv).await;
            }
        });
    }
    Ok(())
}
