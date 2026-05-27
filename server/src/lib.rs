pub mod broadcast;
pub mod client;
pub mod game;
pub mod network_server;
pub mod persistence;
pub mod player;
pub mod server;
pub mod state;
pub mod world;

#[cfg(test)]
mod tests;

use anyhow::Result;
use network::DEFAULT_SERVER_ADDRESS;
use satiscore::log_server;
use server::Server;

pub async fn run_server() -> Result<()> {
    log_server!("Serveur: lancement.");
    let x = String::from(DEFAULT_SERVER_ADDRESS);
    let server = Server::new(&x, "world/test.satis").await?;
    server.run().await
}
