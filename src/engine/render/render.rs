use std::{collections::HashMap, time::Instant};

use cgmath::{EuclideanSpace, Matrix4, Point3, Vector3, dot};
use wgpu::{BindGroup, Buffer, BufferUsages, Device, IndexFormat, Queue, RenderPass, RenderPipeline, Surface, util::{BufferInitDescriptor, DeviceExt}, wgt::BufferDescriptor};

use crate::{
    common::geometry::{plane::Plane, vertex::Vertex},
    engine::render::{camera::{Camera, CameraUniform}, mesh::chunk::ChunkMesh, texture::Texture},
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

    pub world_wireframe_render_pipeline: RenderPipeline,
    pub world_render_pipeline: RenderPipeline,
    pub diffuse_bind_group: BindGroup,
    pub diffuse_texture: Texture,

    pub camera_uniform: CameraUniform,
    pub camera_buffer: Buffer,
    pub camera_bind_group: BindGroup,

    pub gizmo_render_pipeline: RenderPipeline,
    pub gizmo_buffer: Buffer,

    pub wireframe: bool,

    pub chunks: HashMap<(i32, i32, i32), Mesh>,
    pub camera: Camera,
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

pub struct GpuContext {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

pub struct BufferData {
    data: Buffer,
    length: u32,
    capacity: u32
}

impl BufferData {

}

pub struct Mesh {
    vertices: (Buffer, Option<u32>),
    indices: Option<(Buffer, IndexFormat, u32)>,
}

pub struct MeshData {
    vertices: (Vec<Vertex>, Option<u32>),
    indices: Option<(Vec<u32>, IndexFormat, u32)>,
}

impl MeshData {
    pub fn has_index_buffer(&self) -> bool {
        return self.indices.is_some();
    }
}

type MeshId = u32;

pub struct RenderManager {
    context: &'static GpuContext,
    meshes: HashMap<MeshId, Mesh>,
    mesh_pool: Vec<Mesh>,
    id_pool: Vec<MeshId>,
    max_id: MeshId,
}

impl RenderManager {
    pub fn new(
        context: &'static GpuContext,
    ) -> Self {
        Self {
            context: context,
            meshes: HashMap::new(),
            mesh_pool: vec![],
            id_pool: vec![],
            max_id: 0,
        }
    }

    fn get_next_id(&mut self) -> MeshId {
        if let Some(id) = self.id_pool.pop() {
            return id;
        }

        if self.max_id > 0 {
            self.max_id += 1;
        }

        self.max_id
    }

    pub fn update_mesh(&mut self, data: MeshData, id: MeshId) {
        if let Some(mesh) = self.meshes.get_mut(&id) {
            mesh.update(&self.context.queue, data);
        }
    }

    pub fn allocate_mesh(&mut self, data: MeshData) -> MeshId {
        // Mesh pool isn't empty
        let id = self.get_next_id();
        
        if let Some(mut mesh) = self.mesh_pool.pop() {
            mesh.update(&self.context.queue, data);
            self.meshes.insert(id, mesh);
        }
        else {
            // Aucune mesh disponible, on crée

        }
        
        id
    }

    pub fn release_mesh(&mut self, id: MeshId) {
        if let Some(mesh) = self.meshes.remove(&id) {
            self.mesh_pool.push(mesh);
            self.id_pool.push(id);
        }
    }
}

impl Mesh {
    pub fn new(
        device: &Device,
        data: MeshData
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(&data.vertices.0),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        let index_buffer: Option<(Buffer, IndexFormat, u32)> = {
            // Si il y a, alors créer, comme au-dessus
            None
        };
    }

    pub fn get_vertex_buffer(&self) -> &Buffer {
        return &self.vertices.0;
    }

    pub fn get_vertex_count(&self) -> u32 {
        return self.vertices.1.expect("Error:\ntry to get vertex count of a mesh but its value is None.\nMaybe the mesh uses an index buffer?");
    }
    
    pub fn has_index_buffer(&self) -> bool {
        return self.indices.is_some();
    }

    pub fn get_index_buffer(&self) -> &Buffer {
        return &self.indices.as_ref().expect("Error:\ntry to get index buffer of a mesh but its value is None.\nMaybe the mesh is not indexed?").0;
    }

    pub fn get_index_format(&self) -> IndexFormat {
        return self.indices.as_ref().expect("Error:\ntry to get index format of a mesh's index buffer but its value is None.\nMaybe the mesh is not indexed?").1;
    }

    pub fn get_index_count(&self) -> u32 {
        return self.indices.as_ref().expect("Error:\ntry to get index count of a mesh but its value is None.\nMaybe the mesh is not indexed?").2;
    }

    pub fn update(&mut self, queue: &Queue, data: MeshData) {
        // Reste à voir si le buffer a une capacité suffisante pour les données de data, et si non, on le recrée

        // Si le buffer a une taille suffisante, on va écrire les données + (capacité - taille) 0 pour être sûr d'overwrite complètement le buffer
        queue.write_buffer(self.get_vertex_buffer(), 0, bytemuck::cast_slice(&data.vertices.0));
        if let Some(index) = data.indices {
            queue.write_buffer(self.get_index_buffer(), 0, bytemuck::cast_slice(&index.0));
        }
    }
}

impl Renderer {
    pub fn new(
        is_surface_configured: bool,

        world_wireframe_render_pipeline: RenderPipeline,
        world_render_pipeline: RenderPipeline,
        diffuse_bind_group: BindGroup,
        diffuse_texture: Texture,

        camera_uniform: CameraUniform,
        camera_buffer: Buffer,
        camera_bind_group: BindGroup,

        gizmo_render_pipeline: RenderPipeline,
        gizmo_buffer: Buffer,

        dimensions: (u32, u32),
        cam_fovy: f32,
    ) -> Self {
        Self {
            is_surface_configured,

            world_wireframe_render_pipeline,
            world_render_pipeline,
            diffuse_bind_group,
            diffuse_texture,

            camera_uniform,
            camera_buffer,
            camera_bind_group,

            gizmo_render_pipeline,
            gizmo_buffer,

            wireframe: false,

            chunks: HashMap::new(),
            camera: Camera::new(
                Point3::new(0.0, 0.0, 1.0),
                Point3::new(0.0, 0.0, 0.0),
                Vector3::unit_y(),
                (dimensions.0 as f32) / (dimensions.1 as f32),
                cam_fovy,
                0.1,
                1000.0
            )
        }
    }

    pub fn render(&self, surface: &Surface, device: &Device, queue: &Queue) {
        if !self.is_surface_configured {
            return;
        }

        let output = surface.get_current_texture().unwrap();

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.9,
                            g: 0.9,
                            b: 0.9,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            self.render_chunks(&mut render_pass);
            
        }

        // submit will accept anything that implements IntoIter
        queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    pub fn render_chunks(&self, render_pass: &mut RenderPass) {
        let cam_forward = self.camera.forward();
        let cam_eye = self.camera.eye.to_vec();

        let view_proj = self.camera_uniform.get_view_proj();
        let frustum = extract_camera_frustum_planes(view_proj);

        for chunk in &self.chunks {
            // Vertex as Vector3 in the world that is equal to the local origin of the current chunk
            let min: Vector3<f32> = Vector3::new(
                (chunk.0.0) as f32 * CHUNK_SIZE as f32,
                (chunk.0.1) as f32 * CHUNK_SIZE as f32,
                (chunk.0.2) as f32 * CHUNK_SIZE as f32,
            );

            // Vertex as Vector3 in the world that is equal to the absolute opposite of the local origin of the current chunk
            let max = min + Vector3::new(
                CHUNK_SIZE as f32,
                CHUNK_SIZE as f32,
                CHUNK_SIZE as f32
            );

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

            let mesh = chunk.1;
            render_pass.set_vertex_buffer(0, mesh.get_vertex_buffer().slice(..));

            if mesh.has_index_buffer() {
                render_pass.set_index_buffer(mesh.get_index_buffer().slice(..), mesh.get_index_format());
                render_pass.draw_indexed(0..mesh.get_index_count(), 0, 0..1);
            }
            else {
                render_pass.draw(0..mesh.get_vertex_count(), 0..1);
            }
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

// pub fn render_world(render_pass: &mut RenderPass) {
//     if context.renderer.wireframe {
//         render_pass.set_pipeline(&context.renderer.world_wireframe_render_pipeline);
//     }
//     else {
//         render_pass.set_pipeline(&context.renderer.world_render_pipeline);
//     }
    
//     render_pass.set_bind_group(0, &context.renderer.diffuse_bind_group, &[]);
//     render_pass.set_bind_group(1, &context.renderer.camera_bind_group, &[]);

//     let cam_forward = context.game_state.camera.forward();
//     let cam_eye = context.game_state.camera.eye.to_vec();

//     let view_proj = context.renderer.camera_uniform.get_view_proj();
//     let frustum = extract_camera_frustum_planes(view_proj);

//     for chunk_mesh in &context.game_state.world_mesh.meshes {
//         // Check if the vertex buffer is correctly configured before doing any math.
//         let Some(chunk_vertex_buffer) = chunk_mesh.1.buffer.vertex_buffer.as_ref() else {
//             println!("CHUNK RENDERING ERROR: VERTEX BUFFER NOT SET");
//             continue;
//         };
//         let Some(chunk_vertex_number) = chunk_mesh.1.buffer.vertex_number else {
//             println!("CHUNK RENDERING ERROR: VERTEX NUMBER NOT SET");
//             continue;
//         };

//         // Vertex as Vector3 in the world that is equal to the local origin of the current chunk
//         let min = Vector3::new(
//             (chunk_mesh.0 .0) as f32 * CHUNK_SIZE as f32,
//             (chunk_mesh.0 .1) as f32 * CHUNK_SIZE as f32,
//             (chunk_mesh.0 .2) as f32 * CHUNK_SIZE as f32,
//         );
//         // Vertex as Vector3 in the world that is equal to the absolute opposite of the local origin of the current chunk
//         let max = min + Vector3::new(CHUNK_SIZE as f32, CHUNK_SIZE as f32, CHUNK_SIZE as f32);

//         // Check if the chunk is behind the camera.
//         // This allows us to pre-filter chunks before going too further in the tests, at least for chunks that shouldn't be drawn to screen.
//         if is_chunk_behind_camera(&min, &max, &cam_forward, &cam_eye) {
//             continue;
//         }

//         // Check if the chunk is outside the camera's frustum, aka outside it's field of view.
//         // This operation is a little bit more expensive than the one above.
//         // This is why we do the later first : to eliminate chunks as much as possible before doing this final test.
//         if !is_chunk_in_camera_frustum(&min, &max, &frustum) {
//             continue;
//         }

//         render_pass.set_vertex_buffer(0, chunk_vertex_buffer.slice(..));
//         // render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16); // 1.
//         render_pass.draw(0..chunk_vertex_number, 0..1);
//     }
// }

// pub fn render_gizmo(render_pass: &mut RenderPass) {
//     render_pass.set_pipeline(&render_context.renderer.gizmo_render_pipeline);
//     render_pass.set_vertex_buffer(0, render_context.renderer.gizmo_buffer.slice(..));
//     render_pass.draw(0..6, 0..1);
// }