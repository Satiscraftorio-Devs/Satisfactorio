use std::time::Instant;

use cgmath::{EuclideanSpace, InnerSpace, Matrix4, Vector3, dot, num_traits::ToPrimitive};
use wgpu::{Device, Queue};

use crate::{common::geometry::plane::Plane, engine::{core::inputs::InputState, render::{camera::Camera, mesh::world::WorldMesh, render::{RenderManager, Renderer}}}, game::{player::{camera::CameraController, player::Player}, world::{chunk::{CHUNK_SIZE, Chunk}, world::World}}};

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
        player: Player
    ) -> Self {
        Self {
            world,
            world_mesh,
            camera,
            camera_controller,
            player
        }
    }

    pub fn init(&mut self, renderer: &mut Renderer) {
        let world_start = Instant::now();

        let [min_x, max_x, min_y, max_y, min_z, max_z] = self.player.get_rendered_chunk_range();

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    let chunk = Chunk::generate(x, y, z);
                    self.world.set_chunk(x, y, z, chunk);
                }
            }
        }

        println!(
            "Time to make the world: {:.3}ms.",
            world_start.elapsed().as_micros().to_f64().unwrap() / 1_000.0
        );

        let mesh_start = Instant::now();

        self.world_mesh.update(renderer, &self.world, &self.player);

        println!(
            "Time to make meshes: {:.3}ms.",
            mesh_start.elapsed().as_micros().to_f64().unwrap() / 1_000.0
        );
    }
    
    pub fn update(&mut self, renderer: &mut Renderer, inputs: &InputState, dt: f32) {
        let mouse = inputs.get_mouse_delta();
        self.camera_controller.process_keys(inputs);
        self.camera_controller.process_mouse(mouse.0, mouse.1);
        self.player.update(
            dt,
            &mut self.camera,
            &mut self.camera_controller,
            &mut renderer.camera_uniform,
            &mut renderer.camera_buffer,
            &mut renderer.gpu_context.queue
        );

        renderer.render_manager.clear_render_queue();

        let cam_forward = self.camera.forward();
        let cam_eye = self.camera.eye.to_vec();

        let view_proj = renderer.camera_uniform.get_view_proj();
        let frustum = extract_camera_frustum_planes(view_proj);

        for mesh in self.world_mesh.meshes.iter() {

            // Check if the vertex buffer is correctly configured before doing any math.
            // let Some(chunk_vertex_buffer) = chunk_mesh.1.buffer.vertex_buffer.as_ref() else {
            //     println!("CHUNK RENDERING ERROR: VERTEX BUFFER NOT SET");
            //     continue;
            // };
            // let Some(chunk_vertex_number) = chunk_mesh.1.buffer.vertex_number else {
            //     println!("CHUNK RENDERING ERROR: VERTEX NUMBER NOT SET");
            //     continue;
            // };

            // Vertex as Vector3 in the world that is equal to the local origin of the current chunk
            let min = Vector3::new(
                (mesh.0.0) as f32 * CHUNK_SIZE as f32,
                (mesh.0.1) as f32 * CHUNK_SIZE as f32,
                (mesh.0.2) as f32 * CHUNK_SIZE as f32,
            );
            // Vertex as Vector3 in the world that is equal to the absolute opposite of the local origin of the current chunk
            let max = min + Vector3::new(CHUNK_SIZE as f32, CHUNK_SIZE as f32, CHUNK_SIZE as f32);

            // Check if the chunk is behind the camera.
            // This allows us to pre-filter chunks before going too further in the tests, at least for chunks that shouldn't be drawn to screen.
            if is_chunk_behind_camera(&min, &max, &cam_forward, &cam_eye) {
                continue;
            }

            // Check if the chunk is outside the camera's frustum, aka outside it's field of view.
            // This operation is a little bit more expensive than the one above.
            // This is why we do the later first : to eliminate chunks as much as possible before doing this final test.
            if !is_chunk_in_camera_frustum(&min, &max, &frustum) {
                continue;
            }

            renderer.render_manager.mark_mesh_for_rendering(mesh.1.mesh_id.unwrap());
        }
    }
}

fn is_chunk_behind_camera(
    min: &Vector3<f32>,
    max: &Vector3<f32>,
    cam_forward: &Vector3<f32>,
    cam_eye: &Vector3<f32>,
) -> bool {
    let center = min + (max - min) * 0.5;
    let extent = (max - min) * 0.5;

    let radius = extent.x * cam_forward.x.abs()
        + extent.y * cam_forward.y.abs()
        + extent.z * cam_forward.z.abs();

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