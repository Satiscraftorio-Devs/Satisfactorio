use std::{thread::sleep, time::{Duration, Instant}};

use cgmath::{EuclideanSpace, Matrix4, Vector3, dot, num_traits::ToPrimitive};

use crate::{
    common::geometry::plane::Plane, engine::{
        core::application::AppState,
        render::{
            camera::Camera,
            mesh::world::WorldMesh,
            render::{EngineFrameData, GameFrameData, RenderOptions, Renderer},
        },
    }, game::{
        player::{camera::CameraController, player::Player},
        world::{chunk::CHUNK_SIZE_F, world::World},
    }
};
use winit::keyboard::KeyCode;

const FPS_CAP: u32 = 1_000_000;
const DT_CAP: f32 = 1.0 / (FPS_CAP as f32);

pub struct GameState {
    pub world: World,
    pub world_mesh: WorldMesh,
    pub player: Player,
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub delay_ms: f32,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            player: Player::new(),
            world: World::new(),
            world_mesh: WorldMesh::new(),
            camera: Camera::new(cgmath::Point3::new(16.0, 16.0, 16.0), 1.0),
            camera_controller: CameraController::new(32.0, 0.004),
            delay_ms: 0.0
        }
    }
}

impl AppState for GameState {
    fn init(&mut self, renderer: &mut Renderer) {
        self.world.update(&mut renderer.render_manager, &mut self.world_mesh, &self.player);
        self.world_mesh.update(renderer, &self.world, &self.player);
    }

    fn update(&mut self, frame: &EngineFrameData, render_options: &RenderOptions, data: &mut GameFrameData, renderer: &mut Renderer) {

        self.delay_ms += frame.dt;

        if self.delay_ms < DT_CAP {
            sleep(Duration::from_micros(((DT_CAP - self.delay_ms) * 1_000_000.0) as u64));
        }

        self.delay_ms -= DT_CAP;

        // LOGIC
        self.player.update(frame.dt, &mut self.camera, &mut self.camera_controller);

        self.world.update(&mut renderer.render_manager, &mut self.world_mesh, &self.player);
        self.world_mesh.update(renderer, &self.world, &self.player);

        // RENDER
        let view_proj = self.camera.get_view_proj();
        data.camera.update_view_proj(view_proj);

        self.camera.aspect = render_options.aspect;
        let cam_position = self.camera.eye.to_vec();
        let cam_forward = self.camera.forward();
        let cam_frustum = extract_camera_frustum_planes(view_proj);

        let chunks_to_render = self.player.get_rendered_chunk_keys();

        for (key, mesh) in self.world_mesh.meshes.iter() {
            if !chunks_to_render.contains(key) {
                continue;
            }

            let chunk_vector = Vector3::new(CHUNK_SIZE_F, CHUNK_SIZE_F, CHUNK_SIZE_F);
            let min = Vector3::new((key.0) as f32, (key.1) as f32, (key.2) as f32) * CHUNK_SIZE_F;
            let max = min + chunk_vector;

            // First, we check simply if the chunk to render is behind the camera.
            // Second, we check if the chunk is within the field of view of the camera.
            // If any of the above is true, we do not render the chunk.
            // We do the frustum check after the first one because it is more expansive,
            // and the first one would already eliminate ~50% of the chunks very quickly.
            if is_chunk_behind_camera(&min, &max, &cam_forward, &cam_position)
            || !is_chunk_in_camera_frustum(&min, &max, &cam_frustum) {
                continue;
            }

            data.visible_meshes.push(mesh.id.unwrap());
        }
    }

    // Will be used later on for physics and game's systems' logic
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

fn extract_camera_frustum_planes(m: Matrix4<f32>) -> [Plane; 6] {
    [
        Plane { normal: Vector3::new(m[0][3]+m[0][0], m[1][3]+m[1][0], m[2][3]+m[2][0]), d: m[3][3]+m[3][0] }, // left
        Plane { normal: Vector3::new(m[0][3]-m[0][0], m[1][3]-m[1][0], m[2][3]-m[2][0]), d: m[3][3]-m[3][0] }, // right
        Plane { normal: Vector3::new(m[0][3]+m[0][1], m[1][3]+m[1][1], m[2][3]+m[2][1]), d: m[3][3]+m[3][1] }, // bottom
        Plane { normal: Vector3::new(m[0][3]-m[0][1], m[1][3]-m[1][1], m[2][3]-m[2][1]), d: m[3][3]-m[3][1] }, // top
        Plane { normal: Vector3::new(m[0][3]+m[0][2], m[1][3]+m[1][2], m[2][3]+m[2][2]), d: m[3][3]+m[3][2] }, // near
        Plane { normal: Vector3::new(m[0][3]-m[0][2], m[1][3]-m[1][2], m[2][3]-m[2][2]), d: m[3][3]-m[3][2] }, // far
    ].map(|p| p.normalize())
}

fn is_chunk_in_camera_frustum(min: &Vector3<f32>, max: &Vector3<f32>, planes: &[Plane; 6]) -> bool {
    for p in planes {
        let positive = Vector3::new(
            if p.normal.x >= 0.0 { max.x } else { min.x },
            if p.normal.y >= 0.0 { max.y } else { min.y },
            if p.normal.z >= 0.0 { max.z } else { min.z },
        );
        if p.distance(positive) < 0.0 {
            return false;
        }
    }
    true
}
