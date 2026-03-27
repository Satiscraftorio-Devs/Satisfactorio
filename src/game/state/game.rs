use std::time::Instant;

use cgmath::num_traits::ToPrimitive;
use wgpu::{Device, Queue};

use crate::{
    engine::render::{camera::Camera, mesh::world::WorldMesh, render::Renderer},
    game::{
        player::{camera::CameraController, player::Player},
        world::world::World,
    },
};

pub struct GameState {
    pub world: World,
    pub world_mesh: WorldMesh,
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub player: Player,
}

impl GameState {
    pub fn new(
        world: World,
        world_mesh: WorldMesh,
        camera: Camera,
        camera_controller: CameraController,
        player: Player,
    ) -> Self {
        Self {
            world,
            world_mesh,
            camera,
            camera_controller,
            player,
        }
    }

    pub fn init(&mut self, device: &Device) {
        let world_start = Instant::now();

        let chunks_to_rebuild = self.world.update(&self.player);

        println!(
            "Time to make the world: {:.3}ms.",
            world_start.elapsed().as_micros().to_f64().unwrap() / 1_000.0
        );

        let mesh_start = Instant::now();

        self.world_mesh
            .update(device, &mut self.world, &chunks_to_rebuild);

        println!(
            "Time to make meshes: {:.3}ms.",
            mesh_start.elapsed().as_micros().to_f64().unwrap() / 1_000.0
        );
    }

    pub fn update(&mut self, queue: &Queue, renderer: &mut Renderer, device: &Device, dt: f32) {
        self.player.update(
            dt,
            &mut self.camera,
            &mut self.camera_controller,
            &mut renderer.camera_uniform,
            &renderer.camera_buffer,
            queue,
        );

        let chunks_to_rebuild = self.world.update(&self.player);
        self.world_mesh
            .update(device, &mut self.world, &chunks_to_rebuild);
    }
}
