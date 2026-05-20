pub mod engine;
pub mod game;

use winit::event_loop::EventLoop;

use crate::engine::core::application::App;
use crate::game::game::GameState;

pub fn run_client(address: &str) {
    env_logger::init();

    let event_loop = EventLoop::with_user_event().build().expect("Failed starting event loop");
    let game_state = GameState::new(address.to_string());
    let mut app = App::new(game_state);

    event_loop.run_app(&mut app).expect("Failed starting app");
}
