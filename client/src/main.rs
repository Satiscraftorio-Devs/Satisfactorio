mod common;
mod engine;
mod game;

use std::net::Ipv4Addr;

use clap::Parser;
use winit::event_loop::EventLoop;

use crate::{engine::core::application::App, game::game::GameState};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    ip: Ipv4Addr,
    #[arg(short, long)]
    port: u16,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    let event_loop = EventLoop::with_user_event().build().expect("Failed starting event loop");
    let game_state = GameState::new(args.ip, args.port);
    let mut app = App::new(game_state);

    event_loop.run_app(&mut app).expect("Failed starting app");
}
