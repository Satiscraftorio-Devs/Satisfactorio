use std::{collections::HashMap, time::Instant};

use cgmath::{EuclideanSpace, Matrix4, Point3, Vector3, dot};
use wgpu::{BindGroup, Buffer, BufferUsages, Device, IndexFormat, Queue, RenderPass, RenderPipeline, Surface, util::{BufferInitDescriptor, DeviceExt}, wgt::BufferDescriptor};

use crate::{
    common::geometry::{plane::Plane, vertex::{self, Vertex}},
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
    
    pub gpu_context: GpuContext,
    pub render_manager: RenderManager,
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
    capacity: u32,
    format: Option<IndexFormat>,
}

impl BufferData {
    pub fn new(
        data: Buffer,
        length: u32,
        capacity: u32,
        format: Option<IndexFormat>
    ) -> Self {
        Self {
            data,
            length,
            capacity,
            format,
        }
    }
}

pub struct Mesh {
    vertices: BufferData,
    indices: Option<BufferData>,
}

pub struct MeshData {
    /// The vertex buffer data, and the number of vertices if the mesh doesn't use an index buffer (if it does, this value is None since we can get the vertex count from the index buffer).
    vertices: (Vec<Vertex>, u32),
    /// If the mesh uses an index buffer (if it doesn't, this value is None) : the index buffer data, the index format and the number of indices.
    indices: Option<(Vec<u32>, IndexFormat, u32)>,
}

impl MeshData {
    pub fn new(
        vertices: Vec<Vertex>,
        indices: Option<Vec<u32>>
    ) -> Self {
        let vertices = {
            let len = vertices.len() as u32;
            (vertices, len)
        };
        let indices = if let Some(indices) = indices {
            let len = indices.len() as u32;
            Some((indices, IndexFormat::Uint32, len))
        }

        else {
            None
        };

        Self {
            vertices,
            indices,
        }
    }

    pub fn get_vertex_data(&self) -> &Vec<Vertex> {
        return &self.vertices.0;
    }

    pub fn get_vertex_count(&self) -> u32 {
        return self.vertices.1;
    }
    
    pub fn has_index_data(&self) -> bool {
        return self.indices.is_some();
    }

    pub fn get_index_data(&self) -> &Vec<u32> {
        return &self.indices.as_ref().expect("Error:\ntry to get index data of a mesh data but its value is None.\nMaybe the mesh data is not indexed?").0;
    }

    pub fn get_index_format(&self) -> IndexFormat {
        return self.indices.as_ref().expect("Error:\ntry to get index format of a mesh's index buffer but its value is None.\nMaybe the mesh data is not indexed?").1;
    }

    pub fn get_index_count(&self) -> u32 {
        return self.indices.as_ref().expect("Error:\ntry to get index count of a mesh data but its value is None.\nMaybe the mesh data is not indexed?").2;
    }
}

pub type MeshId = u32;

pub struct RenderManager {
    meshes: HashMap<MeshId, Mesh>,
    mesh_pool: Vec<Mesh>,
    id_pool: Vec<MeshId>,
    max_id: MeshId,
    ids_to_render: Vec<MeshId>,
}

impl RenderManager {
    pub fn new() -> Self {
        Self {
            meshes: HashMap::new(),
            mesh_pool: vec![],
            id_pool: vec![],
            max_id: 0,
            ids_to_render: vec![],
        }
    }

    fn get_next_id(&mut self) -> MeshId {
        if let Some(id) = self.id_pool.pop() {
            return id;
        }

        if self.max_id == 0 {
            self.max_id += 1;
            return 0;
        }
        else {
            self.max_id += 1;
            return self.max_id - 1;
        }
    }

    pub fn update_mesh(&mut self, device: &Device, queue: &Queue, data: MeshData, id: MeshId) -> bool {
        if let Some(mesh) = self.meshes.get_mut(&id) {
            mesh.update(device, queue, data);
            return true;
        }
        return false;
    }

    pub fn allocate_mesh(&mut self, device: &Device, queue: &Queue, data: MeshData) -> MeshId {
        let id = self.get_next_id();
        
        let mesh = self.mesh_pool.pop().unwrap_or_else(|| Mesh::new(device, queue, data));
        self.meshes.insert(id, mesh);
        
        println!("Affected mesh with id: {} mesh count: {}", id, self.meshes.len());

        id
    }

    pub fn release_mesh(&mut self, id: MeshId) {
        if let Some(mesh) = self.meshes.remove(&id) {
            self.mesh_pool.push(mesh);
            self.id_pool.push(id);
        }
    }

    pub fn mark_mesh_for_rendering(&mut self, id: MeshId) {
        if self.meshes.contains_key(&id) {
            self.ids_to_render.push(id);
        }
    }

    pub fn get_meshes_to_render(&self) -> Vec<&Mesh> {
        self.ids_to_render.iter().filter_map(|id| self.meshes.get(id)).collect()
    }

    pub fn clear_render_queue(&mut self) {
        self.ids_to_render.clear();
    }
}

const MESH_BUFFER_CAPACITY_MARGIN: f32 = 1.25;

impl Mesh {
    pub fn new(
        device: &Device,
        queue: &Queue,
        data: MeshData
    ) -> Self {
        let vertex_buffer_capacity = (data.get_vertex_data().len() as f32 * MESH_BUFFER_CAPACITY_MARGIN) as u32;
        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Vertex buffer"),
            size: vertex_buffer_capacity as u64 * std::mem::size_of::<Vertex>() as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(
            &vertex_buffer,
            0,
            bytemuck::cast_slice(data.get_vertex_data())
        );
        let vertex_buffer_data = BufferData::new(
            vertex_buffer,
            data.get_vertex_count(),
            vertex_buffer_capacity,
            None
        );

        let mut index_buffer_data: Option<BufferData> = None;

        if data.has_index_data() {
            let index_buffer_capacity = ((data.get_index_count() as f32) * MESH_BUFFER_CAPACITY_MARGIN) as u32;
            let index_buffer = device.create_buffer(&BufferDescriptor {
                label: Some("Index buffer"),
                size: index_buffer_capacity as u64 * std::mem::size_of::<u32>() as u64,
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            queue.write_buffer(
                &index_buffer,
                0,
                bytemuck::cast_slice(&data.get_index_data())
            );
            index_buffer_data = Some(BufferData::new(
                index_buffer,
                data.get_index_count(),
                index_buffer_capacity,
                Some(data.get_index_format())
            ));
        }

        Self {
            vertices: vertex_buffer_data,
            indices: index_buffer_data,
        }
    }

    pub fn get_vertex_buffer(&self) -> &Buffer {
        return &self.vertices.data;
    }

    pub fn get_vertex_count(&self) -> u32 {
        return self.vertices.length;
    }

    pub fn get_vertex_capacity(&self) -> u32 {
        return self.vertices.capacity;
    }
    
    pub fn has_index_buffer(&self) -> bool {
        return self.indices.is_some();
    }

    pub fn get_index_buffer(&self) -> &Buffer {
        return &self.indices.as_ref().expect("Error:\ntry to get index buffer of a mesh but its value is None.\nMaybe the mesh is not indexed?").data;
    }

    pub fn get_index_format(&self) -> IndexFormat {
        return self.indices.as_ref().expect("Error:\ntry to get index format of a mesh's index buffer but the buffer's value is None.\nMaybe the mesh is not indexed?").format.expect("Error:\ntry to get index format of a mesh's index buffer but its value is None.\nMaybe the index buffer is not correctly configured?");
    }

    pub fn get_index_count(&self) -> u32 {
        return self.indices.as_ref().expect("Error:\ntry to get index count of a mesh but its value is None.\nMaybe the mesh is not indexed?").length;
    }

    pub fn get_index_capacity(&self) -> u32 {
        return self.indices.as_ref().expect("Error:\ntry to get index capacity of a mesh but its value is None.\nMaybe the mesh is not indexed?").capacity;
    }

    pub fn update(&mut self, device: &Device, queue: &Queue, data: MeshData) {
        // Reste à voir si le buffer a une capacité suffisante pour les données de data, et si non, on le recrée

        // Si le buffer a une taille suffisante, on va écrire les données + (capacité - taille) 0 pour être sûr d'overwrite complètement le buffer
        if self.get_vertex_capacity() >= data.get_vertex_count() {
            queue.write_buffer(
                self.get_vertex_buffer(),
                0,
                bytemuck::cast_slice(data.get_vertex_data())
            );
        }
        else {
            let vertex_buffer_capacity = (data.get_vertex_data().len() as f32 * MESH_BUFFER_CAPACITY_MARGIN) as u32;

            let vertex_buffer = device.create_buffer(&BufferDescriptor {
                label: Some("Vertex buffer"),
                size: vertex_buffer_capacity as u64 * std::mem::size_of::<Vertex>() as u64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            queue.write_buffer(
                &vertex_buffer,
                0,
                bytemuck::cast_slice(data.get_vertex_data())
            );

            self.vertices = BufferData::new(
                vertex_buffer,
                data.get_vertex_count(),
                vertex_buffer_capacity,
                None
            );
        }

        if data.has_index_data() {
            if self.get_index_capacity() >= data.get_index_count() {
                queue.write_buffer(
                    self.get_index_buffer(),
                    0,
                    bytemuck::cast_slice(data.get_index_data())
                );
            }
            else {
                let indices = data.get_index_data();
                let index_format = data.get_index_format();
                let index_buffer_capacity = ((data.get_index_count() as f32) * MESH_BUFFER_CAPACITY_MARGIN) as u32;

                let index_buffer = device.create_buffer(&BufferDescriptor {
                    label: Some("Index buffer"),
                    size: index_buffer_capacity as u64 * std::mem::size_of::<u32>() as u64,
                    usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                queue.write_buffer(
                    &index_buffer,
                    0,
                    bytemuck::cast_slice(&indices)
                );

                self.indices = Some(BufferData::new(
                    index_buffer,
                    data.get_index_count(),
                    index_buffer_capacity,
                    Some(index_format)
                ));
            }
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

        gpu_context: GpuContext,
        render_manager: RenderManager,
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
            ),

            gpu_context,
            render_manager,
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

            if self.wireframe {
                render_pass.set_pipeline(&self.world_wireframe_render_pipeline);
            }
            else {
                render_pass.set_pipeline(&self.world_render_pipeline);
            }
            
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            // self.render_chunks(&mut render_pass);

            let meshes = self.render_manager.get_meshes_to_render();

            // println!("RENDERING {} MESHES", meshes.len());

            for mesh in meshes {
                // println!("RENDERING MESH");

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

        // submit will accept anything that implements IntoIter
        queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
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