mod engine;
mod common;
mod game;

use std::{cell::RefCell, rc::Rc, sync::{Arc, Mutex}};

use winit::event_loop::EventLoop;

use crate::{engine::{core::application::App, render::{camera::Camera, mesh::world::WorldMesh, render::Renderer}}, game::{player::{camera::CameraController, player::Player}, state::game::GameState, world::world::World}};

fn cull_chunks(renderer: &mut Renderer) {

}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::with_user_event().build().expect("Failed starting event loop");

    // game_state partagé
    let game_state = Rc::new(RefCell::new(None::<GameState>));

    let gs_init = game_state.clone();
    let gs_update = game_state.clone();

    let mut app = App::new(
        move |renderer| {
            let player = Player::new();
            let camera_controller = CameraController::new(128.0, 0.0025);
            let world = World::new();
            let world_mesh = WorldMesh::new();

            let mut gs = GameState::new(
                world,
                world_mesh,
                renderer.camera.clone(),
                camera_controller,
                player,
            );
            gs.init(renderer);
            gs.world_mesh.update(renderer, &gs.world, &gs.player);

            // Stocker dans le RefCell
            *gs_init.borrow_mut() = Some(gs);
        },
        move |dt, renderer, inputs| {
            let mut gs_ref = gs_update.borrow_mut();

            if let Some(gs) = gs_ref.as_mut() {
                gs.update(renderer, inputs, dt);
            }

            renderer.camera = gs_ref.as_ref().unwrap().camera.clone();
        }
    );

    event_loop.run_app(&mut app).expect("Failed starting app");
}

// Tutoriel à voir : https://sotrh.github.io/learn-wgpu/beginner/tutorial7-instancing/
