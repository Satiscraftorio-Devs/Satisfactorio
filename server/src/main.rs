mod broadcast;
mod client;
mod game;
mod network;
mod player;
mod server;
mod state;
mod world;

use anyhow::Result;
use clap::Parser;
use server::Server;
use shared::log_server;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("127.0.0.1:42677"))]
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
