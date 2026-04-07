mod common;
mod engine;
mod game;

use winit::event_loop::EventLoop;

use crate::{engine::core::application::App, game::state::game::GameState};

fn main() {
    env_logger::init();

    let event_loop = EventLoop::with_user_event().build().expect("Failed starting event loop");
    let game_state = GameState::new();
    let mut app = App::new(game_state);

    event_loop.run_app(&mut app).expect("Failed starting app");
}
