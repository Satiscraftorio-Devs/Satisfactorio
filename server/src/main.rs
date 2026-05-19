pub mod broadcast;
pub mod client;
pub mod game;
pub mod network;
pub mod player;
pub mod server;
pub mod state;
pub mod world;

#[cfg(test)]
mod tests;

use anyhow::Result;
use clap::Parser;
use server::Server;
use shared::log_server;
use shared::network::DEFAULT_SERVER_ADDRESS;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from(DEFAULT_SERVER_ADDRESS))]
    address: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    log_server!("Serveur: lancement.");
    let args = Args::parse();
    let server = Server::new(&args.address).await?;
    server.state().init_random_seed();
    server.run().await
}
