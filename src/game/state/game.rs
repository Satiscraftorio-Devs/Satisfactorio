use std::time::Instant;

use cgmath::{dot, num_traits::ToPrimitive, EuclideanSpace, Matrix4, Vector3};

use crate::{
    common::geometry::plane::Plane,
    engine::{
        core::{application::AppState, inputs::InputState},
        render::{
            mesh::world::WorldMesh,
            render::{EngineFrameData, GameFrameData, RenderOptions, Renderer},
        },
    },
    game::{
        player::player::Player,
        world::{
            chunk::{Chunk, CHUNK_SIZE_F},
            world::World,
        },
    },
};

pub struct GameState {
    pub world: World,
    pub world_mesh: WorldMesh,
    pub player: Player,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            player: Player::new(),
            world: World::new(),
            world_mesh: WorldMesh::new(),
        }
    }
}

impl AppState for GameState {
    fn init(&mut self, renderer: &mut Renderer) {
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

    fn update(&mut self, frame: &EngineFrameData, inputs: &InputState, render_options: &RenderOptions, data: &mut GameFrameData) {
        self.player.update(frame.dt, inputs);

        let view_proj = self.player.camera.get_view_proj(render_options);
        let cam_forward = self.player.camera.forward();
        let cam_position = self.player.camera.position.to_vec();
        let frustum = extract_camera_frustum_planes(view_proj);

        for mesh in self.world_mesh.meshes.iter() {
            // Represents the vector that goes from the world origin to the chunk's opposite boundaries
            let chunk_vector = Vector3::new(CHUNK_SIZE_F, CHUNK_SIZE_F, CHUNK_SIZE_F);

            // Vertex as Vector3 in the world that is equal to the local origin of the current chunk
            let min = Vector3::new((mesh.0 .0) as f32, (mesh.0 .1) as f32, (mesh.0 .2) as f32) * CHUNK_SIZE_F;

            // Vertex as Vector3 in the world that is equal to the absolute opposite of the local origin of the current chunk
            let max = min + chunk_vector;

            // Check if the chunk is behind the camera.
            // This allows us to pre-filter chunks before going too further in the tests, at least for chunks that shouldn't be drawn to screen.
            if is_chunk_behind_camera(&min, &max, &cam_forward, &cam_position) {
                continue;
            }

            // Check if the chunk is outside the camera's frustum, aka outside it's field of view.
            // This operation is a little bit more expensive than the one above.
            // This is why we do the later first : to eliminate chunks as much as possible before doing this final test.
            if !is_chunk_in_camera_frustum(&min, &max, &frustum) {
                continue;
            }

            data.visible_meshes.push(mesh.1.mesh_id.unwrap());
        }

        data.camera.update_view_proj(view_proj);
    }

    fn fixed_update(
        &mut self,
        _frame: &EngineFrameData,
        _inputs: &InputState,
        _render_options: &RenderOptions,
        _data: &mut GameFrameData,
    ) {
        // ...
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
        Plane {
            normal: Vector3::new(m[0][3] + m[0][0], m[1][3] + m[1][0], m[2][3] + m[2][0]),
            d: m[3][3] + m[3][0],
        }, // left
        Plane {
            normal: Vector3::new(m[0][3] - m[0][0], m[1][3] - m[1][0], m[2][3] - m[2][0]),
            d: m[3][3] - m[3][0],
        }, // right
        Plane {
            normal: Vector3::new(m[0][3] + m[0][1], m[1][3] + m[1][1], m[2][3] + m[2][1]),
            d: m[3][3] + m[3][1],
        }, // bottom
        Plane {
            normal: Vector3::new(m[0][3] - m[0][1], m[1][3] - m[1][1], m[2][3] - m[2][1]),
            d: m[3][3] - m[3][1],
        }, // top
        Plane {
            normal: Vector3::new(m[0][3] + m[0][2], m[1][3] + m[1][2], m[2][3] + m[2][2]),
            d: m[3][3] + m[3][2],
        }, // near
        Plane {
            normal: Vector3::new(m[0][3] - m[0][2], m[1][3] - m[1][2], m[2][3] - m[2][2]),
            d: m[3][3] - m[3][2],
        }, // far
    ]
    .map(|p| p.normalize())
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
