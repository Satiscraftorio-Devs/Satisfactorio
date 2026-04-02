use std::{collections::HashMap, time::Instant};

use rayon::vec;
use wgpu::{wgt::BufferDescriptor, BindGroup, Buffer, BufferUsages, Device, IndexFormat, Queue, RenderPipeline, TextureView};

use crate::{
    common::geometry::vertex::Vertex,
    engine::render::text::TextRenderer,
    engine::render::{camera::RenderCamera, texture::Texture},
};

pub struct EngineFrameData {
    pub dt: f32,
    pub fps: u32,
    pub fps_timer: f32,
    pub last_frame: Instant,
    pub frame_count: u32,
}

pub struct GameFrameData {
    pub camera: RenderCamera,
    pub visible_meshes: Vec<MeshId>,
}

impl GameFrameData {
    pub fn blank() -> Self {
        Self {
            camera: RenderCamera::new(),
            visible_meshes: vec![],
        }
    }

    pub fn reset(&mut self) {
        self.camera = RenderCamera::new();
        self.visible_meshes.clear();
    }
}

pub struct RenderOptions {
    pub aspect: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl RenderOptions {
    pub fn new(aspect: f32, znear: f32, zfar: f32) -> Self {
        Self { aspect, znear, zfar }
    }
}

pub struct Renderer {
    pub is_surface_configured: bool,

    pub world_wireframe_render_pipeline: RenderPipeline,
    pub world_render_pipeline: RenderPipeline,
    pub diffuse_bind_group: BindGroup,
    pub diffuse_texture: Texture,

    pub camera_buffer: Buffer,
    pub camera_bind_group: BindGroup,

    pub gizmo_render_pipeline: RenderPipeline,
    pub gizmo_buffer: Buffer,

    pub wireframe: bool,

    pub chunks: HashMap<(i32, i32, i32), Mesh>,

    pub gpu_context: GpuContext,
    pub render_manager: RenderManager,

    pub render_options: RenderOptions,

    pub depth_texture: wgpu::Texture,
    pub depth_view: TextureView,
}

impl EngineFrameData {
    pub fn new() -> Self {
        Self {
            dt: 0.0,
            fps: 0,
            fps_timer: 0.0,
            last_frame: Instant::now(),
            frame_count: 0,
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
    pub fn new(data: Buffer, length: u32, capacity: u32, format: Option<IndexFormat>) -> Self {
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
    vertices: (Vec<Vertex>, u32),
    indices: Option<(Vec<u32>, IndexFormat, u32)>,
}

impl MeshData {
    pub fn new(vertices: Vec<Vertex>, indices: Option<Vec<u32>>) -> Self {
        let vertices = {
            let len = vertices.len() as u32;
            (vertices, len)
        };
        let indices = if let Some(indices) = indices {
            let len = indices.len() as u32;
            Some((indices, IndexFormat::Uint32, len))
        } else {
            None
        };

        Self { vertices, indices }
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
        return &self
            .indices
            .as_ref()
            .expect("Error:\ntry to get index data of a mesh data but its value is None.\nMaybe the mesh data is not indexed?")
            .0;
    }

    pub fn get_index_format(&self) -> IndexFormat {
        return self
            .indices
            .as_ref()
            .expect(
                "Error:\ntry to get index format of a mesh's index buffer but its value is None.\nMaybe the mesh data is not indexed?",
            )
            .1;
    }

    pub fn get_index_count(&self) -> u32 {
        return self
            .indices
            .as_ref()
            .expect("Error:\ntry to get index count of a mesh data but its value is None.\nMaybe the mesh data is not indexed?")
            .2;
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
        } else {
            self.max_id += 1;
            return self.max_id - 1;
        }
    }

    pub fn allocate_mesh(&mut self, device: &Device, queue: &Queue, data: MeshData) -> MeshId {
        let id = self.get_next_id();

        let mesh = {
            if let Some(mut mesh) = self.mesh_pool.pop() {
                mesh.update(device, queue, data);
                mesh
            } else {
                Mesh::new(device, queue, data)
            }
        };
        self.meshes.insert(id, mesh);

        println!("Affected mesh with id: {} mesh count: {}", id, self.meshes.len());

        id
    }

    pub fn update_mesh(&mut self, device: &Device, queue: &Queue, data: MeshData, id: MeshId) -> bool {
        if let Some(mesh) = self.meshes.get_mut(&id) {
            mesh.update(device, queue, data);
            return true;
        }
        return false;
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
    pub fn new(device: &Device, queue: &Queue, data: MeshData) -> Self {
        let vertex_buffer_capacity = (data.get_vertex_data().len() as f32 * MESH_BUFFER_CAPACITY_MARGIN) as u32;
        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Vertex buffer"),
            size: vertex_buffer_capacity as u64 * std::mem::size_of::<Vertex>() as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(data.get_vertex_data()));
        let vertex_buffer_data = BufferData::new(vertex_buffer, data.get_vertex_count(), vertex_buffer_capacity, None);

        let mut index_buffer_data: Option<BufferData> = None;

        if data.has_index_data() {
            let index_buffer_capacity = ((data.get_index_count() as f32) * MESH_BUFFER_CAPACITY_MARGIN) as u32;
            let index_buffer = device.create_buffer(&BufferDescriptor {
                label: Some("Index buffer"),
                size: index_buffer_capacity as u64 * std::mem::size_of::<u32>() as u64,
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            queue.write_buffer(&index_buffer, 0, bytemuck::cast_slice(&data.get_index_data()));
            index_buffer_data = Some(BufferData::new(
                index_buffer,
                data.get_index_count(),
                index_buffer_capacity,
                Some(data.get_index_format()),
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

    pub fn set_vertex_count(&mut self, count: u32) {
        self.vertices.length = count;
    }

    pub fn set_vertex_capacity(&mut self, capacity: u32) {
        self.vertices.capacity = capacity;
    }

    pub fn has_index_buffer(&self) -> bool {
        return self.indices.is_some();
    }

    pub fn get_index_buffer(&self) -> &Buffer {
        return &self
            .indices
            .as_ref()
            .expect("Error:\ntry to get index buffer of a mesh but its value is None.\nMaybe the mesh is not indexed?")
            .data;
    }

    pub fn get_index_format(&self) -> IndexFormat {
        return self.indices.as_ref().expect("Error:\ntry to get index format of a mesh's index buffer but the buffer's value is None.\nMaybe the mesh is not indexed?").format.expect("Error:\ntry to get index format of a mesh's index buffer but its value is None.\nMaybe the index buffer is not correctly configured?");
    }

    pub fn get_index_count(&self) -> u32 {
        return self
            .indices
            .as_ref()
            .expect("Error:\ntry to get index count of a mesh but its value is None.\nMaybe the mesh is not indexed?")
            .length;
    }

    pub fn get_index_capacity(&self) -> u32 {
        return self
            .indices
            .as_ref()
            .expect("Error:\ntry to get index capacity of a mesh but its value is None.\nMaybe the mesh is not indexed?")
            .capacity;
    }

    pub fn set_index_count(&mut self, count: u32) {
        if self.indices.is_none() {
            return;
        }
        self.indices.as_mut().unwrap().length = count;
    }

    pub fn set_index_capacity(&mut self, capacity: u32) {
        if self.indices.is_none() {
            return;
        }
        self.indices.as_mut().unwrap().capacity = capacity;
    }

    pub fn update(&mut self, device: &Device, queue: &Queue, data: MeshData) {
        if self.get_vertex_capacity() >= data.get_vertex_count() {
            queue.write_buffer(self.get_vertex_buffer(), 0, bytemuck::cast_slice(data.get_vertex_data()));
            self.set_vertex_count(data.get_vertex_count());
        } else {
            self.get_vertex_buffer().destroy();

            let vertex_buffer_capacity = (data.get_vertex_data().len() as f32 * MESH_BUFFER_CAPACITY_MARGIN) as u32;

            let vertex_buffer = device.create_buffer(&BufferDescriptor {
                label: Some("Vertex buffer"),
                size: vertex_buffer_capacity as u64 * std::mem::size_of::<Vertex>() as u64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            queue.write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(data.get_vertex_data()));

            self.set_vertex_capacity(vertex_buffer_capacity);
            self.set_vertex_count(data.get_vertex_count());

            self.vertices = BufferData::new(vertex_buffer, data.get_vertex_count(), vertex_buffer_capacity, None);
        }

        if data.has_index_data() {
            if self.get_index_capacity() >= data.get_index_count() {
                queue.write_buffer(self.get_index_buffer(), 0, bytemuck::cast_slice(data.get_index_data()));
                self.set_index_count(data.get_index_count());
            } else {
                self.get_index_buffer().destroy();

                let indices = data.get_index_data();
                let index_format = data.get_index_format();
                let index_buffer_capacity = ((data.get_index_count() as f32) * MESH_BUFFER_CAPACITY_MARGIN) as u32;

                let index_buffer = device.create_buffer(&BufferDescriptor {
                    label: Some("Index buffer"),
                    size: index_buffer_capacity as u64 * std::mem::size_of::<u32>() as u64,
                    usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                queue.write_buffer(&index_buffer, 0, bytemuck::cast_slice(&indices));

                self.set_index_capacity(index_buffer_capacity);
                self.set_index_count(data.get_index_count());

                self.indices = Some(BufferData::new(
                    index_buffer,
                    data.get_index_count(),
                    index_buffer_capacity,
                    Some(index_format),
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

        camera_buffer: Buffer,
        camera_bind_group: BindGroup,

        gizmo_render_pipeline: RenderPipeline,
        gizmo_buffer: Buffer,

        dimensions: (u32, u32),

        gpu_context: GpuContext,
        render_manager: RenderManager,

        depth_texture: wgpu::Texture,
        depth_view: TextureView,
    ) -> Self {
        Self {
            is_surface_configured,

            world_wireframe_render_pipeline,
            world_render_pipeline,
            diffuse_bind_group,
            diffuse_texture,

            camera_buffer,
            camera_bind_group,

            gizmo_render_pipeline,
            gizmo_buffer,

            wireframe: false,

            chunks: HashMap::new(),

            gpu_context,
            render_manager,

            render_options: RenderOptions::new((dimensions.0 as f32) / (dimensions.1 as f32), 0.1, 1000.0),

            depth_texture,
            depth_view,
        }
    }

    pub fn render<'a>(&'a mut self, camera: &RenderCamera, text_renderer: Option<&'a mut TextRenderer>) {
        if !self.is_surface_configured {
            return;
        }

        let surface = &self.gpu_context.surface;
        let device = &self.gpu_context.device;
        let queue = &self.gpu_context.queue;

        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&camera.get_view_proj_raw()));

        let output = surface.get_current_texture().unwrap();

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            if self.wireframe {
                render_pass.set_pipeline(&self.world_wireframe_render_pipeline);
            } else {
                render_pass.set_pipeline(&self.world_render_pipeline);
            }

            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            let meshes = self.render_manager.get_meshes_to_render();

            let mut rendered_mesh_count = meshes.len();

            // println!("Rendering {} meshes", meshes.len());

            for mesh in meshes {
                if mesh.get_vertex_count() == 0 || mesh.get_vertex_capacity() == 0 {
                    rendered_mesh_count -= 1;
                    continue;
                }

                render_pass.set_vertex_buffer(0, mesh.get_vertex_buffer().slice(..));

                if mesh.has_index_buffer() {
                    render_pass.set_index_buffer(mesh.get_index_buffer().slice(..), mesh.get_index_format());
                    render_pass.draw_indexed(0..mesh.get_index_count(), 0, 0..1);
                } else {
                    render_pass.draw(0..mesh.get_vertex_count(), 0..1);
                }
            }

            render_pass.set_pipeline(&self.gizmo_render_pipeline);
            render_pass.set_vertex_buffer(0, self.gizmo_buffer.slice(..));
            render_pass.draw(0..6, 0..1);
        }

        if let Some(text_renderer) = text_renderer {
            let mut text_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Text Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            text_renderer.render(device, queue, &mut text_render_pass);
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.render_manager.clear_render_queue();
    }
}
