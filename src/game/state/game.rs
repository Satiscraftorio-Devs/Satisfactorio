use std::time::Instant;

use cgmath::{dot, num_traits::ToPrimitive, EuclideanSpace, Vector3};

use crate::{
    engine::{
        core::application::AppState,
        render::{
            camera::Camera,
            mesh::world::WorldMesh,
            render::{EngineFrameData, GameFrameData, RenderOptions, Renderer},
        },
    },
    game::{
        player::{camera::CameraController, player::Player},
        world::{chunk::CHUNK_SIZE_F, world::World},
    },
};
use winit::keyboard::KeyCode;

pub struct GameState {
    pub world: World,
    pub world_mesh: WorldMesh,
    pub player: Player,
    pub camera: Camera,
    pub camera_controller: CameraController,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            player: Player::new(),
            world: World::new(),
            world_mesh: WorldMesh::new(),
            camera: Camera::new(cgmath::Point3::new(16.0, 16.0, 16.0), 1.0),
            camera_controller: CameraController::new(32.0, 0.004),
        }
    }
}

impl AppState for GameState {
    fn init(&mut self, renderer: &mut Renderer) {
        let world_start = Instant::now();

        let chunks_to_rebuild = self.world.update(&self.player);

        println!(
            "Time to make the world: {:.3}ms.",
            world_start.elapsed().as_micros().to_f64().unwrap() / 1_000.0
        );

        let mesh_start = Instant::now();

        self.world_mesh.update(renderer, &self.world, &self.player, &chunks_to_rebuild);

        println!(
            "Time to make meshes: {:.3}ms.",
            mesh_start.elapsed().as_micros().to_f64().unwrap() / 1_000.0
        );
    }

    fn update(&mut self, frame: &EngineFrameData, render_options: &RenderOptions, data: &mut GameFrameData, renderer: &mut Renderer) {
        self.player.update(frame.dt, &mut self.camera, &mut self.camera_controller);

        self.camera.aspect = render_options.aspect;

        let chunks_to_rebuild = self.world.update(&self.player);
        self.world_mesh.update(renderer, &self.world, &self.player, &chunks_to_rebuild);

        let view_proj = self.camera.get_view_proj();
        data.camera.update_view_proj(view_proj);

        let cam_position = self.camera.eye.to_vec();
        let cam_forward = self.camera.forward();

        for mesh in self.world_mesh.meshes.iter() {
            let chunk_vector = Vector3::new(CHUNK_SIZE_F, CHUNK_SIZE_F, CHUNK_SIZE_F);

            let min = Vector3::new((mesh.0 .0) as f32, (mesh.0 .1) as f32, (mesh.0 .2) as f32) * CHUNK_SIZE_F;

            let max = min + chunk_vector;

            if is_chunk_behind_camera(&min, &max, &cam_forward, &cam_position) {
                continue;
            }

            data.visible_meshes.push(mesh.1.mesh_id.unwrap());
        }
    }

    fn fixed_update(&mut self, _frame: &EngineFrameData, _render_options: &RenderOptions, _data: &mut GameFrameData) {}

    fn on_mouse_move(&mut self, dx: f64, dy: f64) {
        self.camera_controller.process_mouse(dx, dy);
    }

    fn on_key(&mut self, code: KeyCode, is_pressed: bool) {
        self.camera_controller.handle_key(code, is_pressed);
    }
}

fn is_chunk_behind_camera(min: &Vector3<f32>, max: &Vector3<f32>, cam_forward: &Vector3<f32>, cam_eye: &Vector3<f32>) -> bool {
    let center = min + (max - min) * 0.5;
    let extent = (max - min) * 0.5;

    let radius = extent.x * cam_forward.x.abs() + extent.y * cam_forward.y.abs() + extent.z * cam_forward.z.abs();

    let distance = dot(*cam_forward, center - *cam_eye);

    distance + radius < 0.0
}
