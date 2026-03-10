use std::time::Instant;

use cgmath::{dot, EuclideanSpace, Matrix4, Vector3};
use wgpu::{BindGroup, Buffer, RenderPass, RenderPipeline};

use crate::{
    common::geometry::plane::Plane,
    engine::render::{camera::CameraUniform, texture::Texture},
    game::{state::game::GameState, world::chunk::CHUNK_SIZE},
};

pub struct FrameData {
    pub dt: f32,
    pub fps: u32,
    pub fps_timer: f32,
    pub last_frame: Instant,
    pub frame_count: u32,
}

pub struct Renderer {
    pub is_surface_configured: bool,

    pub world_render_pipeline: RenderPipeline,
    pub diffuse_bind_group: BindGroup,
    pub diffuse_texture: Texture,

    pub camera_uniform: CameraUniform,
    pub camera_buffer: Buffer,
    pub camera_bind_group: BindGroup,

    pub gizmo_render_pipeline: RenderPipeline,
    pub gizmo_buffer: Buffer,
}

pub(crate) struct RenderContext<'a> {
    pub game_state: &'a GameState,
    pub frame_data: &'a FrameData,
    pub renderer: &'a Renderer,
}

impl FrameData {
    pub fn new(dt: f32, fps: u32, fps_timer: f32, last_frame: Instant, frame_count: u32) -> Self {
        Self {
            dt,
            fps,
            fps_timer,
            last_frame,
            frame_count,
        }
    }
}

impl Renderer {
    pub fn new(
        is_surface_configured: bool,

        world_render_pipeline: RenderPipeline,
        diffuse_bind_group: BindGroup,
        diffuse_texture: Texture,

        camera_uniform: CameraUniform,
        camera_buffer: Buffer,
        camera_bind_group: BindGroup,

        gizmo_render_pipeline: RenderPipeline,
        gizmo_buffer: Buffer,
    ) -> Self {
        Self {
            is_surface_configured,

            world_render_pipeline,
            diffuse_bind_group,
            diffuse_texture,

            camera_uniform,
            camera_buffer,
            camera_bind_group,

            gizmo_render_pipeline,
            gizmo_buffer,
        }
    }
}

impl<'a> RenderContext<'a> {
    pub fn new(
        frame_data: &'a FrameData,
        game_state: &'a GameState,
        renderer: &'a Renderer,
    ) -> Self {
        Self {
            game_state,
            frame_data,
            renderer,
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

pub fn render_world(render_pass: &mut RenderPass, context: &RenderContext) {
    render_pass.set_pipeline(&context.renderer.world_render_pipeline);
    render_pass.set_bind_group(0, &context.renderer.diffuse_bind_group, &[]);
    render_pass.set_bind_group(1, &context.renderer.camera_bind_group, &[]);

    let cam_forward = context.game_state.camera.forward();
    let cam_eye = context.game_state.camera.eye.to_vec();

    let view_proj = context.renderer.camera_uniform.get_view_proj();
    let frustum = extract_camera_frustum_planes(view_proj);

    for chunk_mesh in &context.game_state.world_mesh.meshes {
        // Check if the vertex buffer is correctly configured before doing any math.
        let Some(chunk_vertex_buffer) = chunk_mesh.1.buffer.vertex_buffer.as_ref() else {
            println!("CHUNK RENDERING ERROR: VERTEX BUFFER NOT SET");
            continue;
        };
        let Some(chunk_vertex_number) = chunk_mesh.1.buffer.vertex_number else {
            println!("CHUNK RENDERING ERROR: VERTEX NUMBER NOT SET");
            continue;
        };

        // Vertex as Vector3 in the world that is equal to the local origin of the current chunk
        let min = Vector3::new(
            (chunk_mesh.0 .0) as f32 * CHUNK_SIZE as f32,
            (chunk_mesh.0 .1) as f32 * CHUNK_SIZE as f32,
            (chunk_mesh.0 .2) as f32 * CHUNK_SIZE as f32,
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

        render_pass.set_vertex_buffer(0, chunk_vertex_buffer.slice(..));
        // render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16); // 1.
        render_pass.draw(0..chunk_vertex_number, 0..1);
    }
}

pub fn render_gizmo(render_pass: &mut RenderPass, render_context: &RenderContext) {
    render_pass.set_pipeline(&render_context.renderer.gizmo_render_pipeline);
    render_pass.set_vertex_buffer(0, render_context.renderer.gizmo_buffer.slice(..));
    render_pass.draw(0..6, 0..1);
}
