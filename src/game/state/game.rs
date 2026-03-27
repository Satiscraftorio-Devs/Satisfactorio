use std::time::Instant;

use cgmath::{num_traits::ToPrimitive, InnerSpace, Vector3};
use wgpu::{Device, Queue};

use crate::{
    engine::render::{camera::Camera, mesh::world::WorldMesh, render::Renderer},
    game::{
        player::{camera::CameraController, player::Player},
        world::{chunk::Chunk, world::World},
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

        let [min_x, max_x, min_y, max_y, min_z, max_z] = self.player.get_rendered_chunk_range();

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    let chunk = Chunk::generate(x, y, z, &self.world.perlin);
                    self.world.set_chunk(x, y, z, chunk);
                }
            }
        }

        println!(
            "Time to make the world: {:.3}ms.",
            world_start.elapsed().as_micros().to_f64().unwrap() / 1_000.0
        );

        let mesh_start = Instant::now();

        self.world_mesh.update(&device, &self.world, &self.player);

        println!(
            "Time to make meshes: {:.3}ms.",
            mesh_start.elapsed().as_micros().to_f64().unwrap() / 1_000.0
        );
    }

    pub fn update(&mut self, queue: &Queue, renderer: &mut Renderer, dt: f32) {
        self.player.update(
            dt,
            &mut self.camera,
            &mut self.camera_controller,
            &mut renderer.camera_uniform,
            &renderer.camera_buffer,
            queue,
        );
    }
}
