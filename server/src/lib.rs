pub mod broadcast;
pub mod client;
pub mod game;
pub mod network;
pub mod player;
pub mod server;
pub mod state;
pub mod world;

use anyhow::Result;
use server::Server;
use shared::log_server;
use shared::network::DEFAULT_SERVER_ADDRESS;

pub async fn run_server() -> Result<()> {
    log_server!("Serveur: lancement.");
    let x = String::from(DEFAULT_SERVER_ADDRESS);
    let server = Server::new(&x).await?;
    server.state().init_random_seed();
    server.run().await
}
