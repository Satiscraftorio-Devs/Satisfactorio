use std::{sync::Arc, time::Instant};

use shared::world::data::chunk::CHUNK_SIZE_F;
use wgpu::{
    wgt::{CommandEncoderDescriptor, DeviceDescriptor, DrawIndirectArgs},
    Adapter, Backends, BindGroup, Buffer, CommandEncoder, Device, ExperimentalFeatures, Features, Instance, InstanceDescriptor, Limits,
    PowerPreference, PresentMode, Queue, RenderPipeline, RequestAdapterOptions, Surface, SurfaceConfiguration, Texture, TextureUsages,
    TextureView, Trace,
};
use winit::window::Window;

use crate::{
    common::geometry::vertex::Vertex,
    engine::{
        core::gpu::pipeline::Pipelines,
        render::{camera::RenderCamera, manager::RenderManager, text::TextRenderer, texture::TextureManager},
    },
};

const WIREFRAME: bool = false;
const SHOW_CHUNK_BORDERS: bool = false;

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

    pub pipelines: Pipelines,

    pub diffuse_bind_group: BindGroup,
    pub diffuse_texture_array: TextureManager,

    pub camera_buffer: Buffer,
    pub camera_bind_group: BindGroup,

    pub gizmo_render_pipeline: RenderPipeline,
    pub gizmo_buffer: Buffer,

    pub wireframe: bool,
    pub show_chunk_borders: bool,

    pub chunk_borders_buffer: Buffer,

    pub gpu_context: GpuContext,
    pub render_manager: RenderManager,

    pub render_options: RenderOptions,

    pub depth_texture: Texture,
    pub depth_view: TextureView,

    pub frame_encoder: Option<CommandEncoder>,
}

pub struct GpuResources {
    device: Device,
    queue: Queue,
}

impl GpuResources {
    pub fn new(device: Device, queue: Queue) -> Self {
        Self { device, queue }
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn device_mut(&mut self) -> &mut Device {
        &mut self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn queue_mut(&mut self) -> &mut Queue {
        &mut self.queue
    }

    pub fn device_queue(&self) -> (&Device, &Queue) {
        (&self.device, &self.queue)
    }

    pub fn device_queue_mut(&mut self) -> (&mut Device, &mut Queue) {
        (&mut self.device, &mut self.queue)
    }
}

pub struct GpuContext {
    pub surface: Surface<'static>,
    pub resources: GpuResources,
    pub config: SurfaceConfiguration,
    pub limits: Limits,
    pub features: Features,
}

impl From<&GpuContext> for GpuResources {
    fn from(ctx: &GpuContext) -> Self {
        Self {
            device: ctx.resources.device().clone(),
            queue: ctx.resources.queue().clone(),
        }
    }
}

impl GpuContext {
    pub fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();
        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))?;

        let features = {
            let mut requested = vec![
                Features::CONSERVATIVE_RASTERIZATION,
                Features::POLYGON_MODE_LINE,
                Features::MULTI_DRAW_INDIRECT_COUNT,
            ];

            requested.retain(|value| adapter.features().contains(*value));
            let result = requested.iter().fold(Features::empty(), |acc, value| acc.union(*value));
            result
        };

        let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor {
            label: None,
            required_features: features,
            experimental_features: ExperimentalFeatures::disabled(),
            required_limits: Limits::default(),
            memory_hints: Default::default(),
            trace: Trace::Off,
        }))?;

        let resources = GpuResources::new(device, queue);

        let limits = resources.device().limits();
        let features = resources.device().features().intersection(adapter.features());

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::AutoNoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        Ok(Self {
            surface,
            resources,
            config,
            limits,
            features,
        })
    }
}

impl Renderer {
    pub fn new(
        is_surface_configured: bool,

        pipelines: Pipelines,

        diffuse_bind_group: BindGroup,
        texture_manager: TextureManager,

        camera_buffer: Buffer,
        camera_bind_group: BindGroup,

        gizmo_render_pipeline: RenderPipeline,
        gizmo_buffer: Buffer,

        chunk_borders_buffer: Buffer,

        gpu_context: GpuContext,
        render_manager: RenderManager,

        render_options: RenderOptions,

        depth_texture: Texture,
        depth_view: TextureView,
    ) -> Self {
        let frame_encoder = gpu_context.resources.device().create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Frame encoder"),
        });
        Self {
            is_surface_configured,

            pipelines,

            diffuse_bind_group,
            diffuse_texture_array: texture_manager,

            camera_buffer,
            camera_bind_group,

            gizmo_render_pipeline,
            gizmo_buffer,

            wireframe: WIREFRAME,
            show_chunk_borders: SHOW_CHUNK_BORDERS,

            chunk_borders_buffer,

            gpu_context,
            render_manager,

            render_options,

            depth_texture,
            depth_view,

            frame_encoder: Some(frame_encoder),
        }
    }

    pub fn render<'a>(&'a mut self, camera: &RenderCamera, text_renderer: Option<&'a mut TextRenderer>) {
        let chunk_borders = vec![
            Vertex::new_with_rgba(0.0, 0.0, 0.0, 0, 255, 0, 255, 0, 3.0, 0.0, 1.0),
            Vertex::new_with_rgba(0.0, CHUNK_SIZE_F, 0.0, 0, 255, 0, 255, 0, 3.0, 0.0, 1.0),
            Vertex::new_with_rgba(0.0, 0.0, CHUNK_SIZE_F, 0, 255, 0, 255, 0, 3.0, 0.0, 1.0),
            Vertex::new_with_rgba(0.0, CHUNK_SIZE_F, CHUNK_SIZE_F, 0, 255, 0, 255, 0, 3.0, 0.0, 1.0),
            Vertex::new_with_rgba(CHUNK_SIZE_F, 0.0, 0.0, 0, 255, 0, 255, 0, 3.0, 0.0, 1.0),
            Vertex::new_with_rgba(CHUNK_SIZE_F, CHUNK_SIZE_F, 0.0, 0, 255, 0, 255, 0, 3.0, 0.0, 1.0),
            Vertex::new_with_rgba(CHUNK_SIZE_F, 0.0, CHUNK_SIZE_F, 0, 255, 0, 255, 0, 3.0, 0.0, 1.0),
            Vertex::new_with_rgba(CHUNK_SIZE_F, CHUNK_SIZE_F, CHUNK_SIZE_F, 0, 255, 0, 255, 0, 3.0, 0.0, 1.0),
            Vertex::new_with_rgba(0.0, 0.0, 0.0, 255, 0, 0, 255, 0, 3.0, 0.0, 1.0),
            Vertex::new_with_rgba(CHUNK_SIZE_F, 0.0, 0.0, 255, 0, 0, 255, 0, 3.0, 0.0, 1.0),
            Vertex::new_with_rgba(0.0, 0.0, CHUNK_SIZE_F, 255, 0, 0, 255, 0, 3.0, 0.0, 1.0),
            Vertex::new_with_rgba(CHUNK_SIZE_F, 0.0, CHUNK_SIZE_F, 255, 0, 0, 255, 0, 3.0, 0.0, 1.0),
            Vertex::new_with_rgba(0.0, CHUNK_SIZE_F, 0.0, 255, 0, 0, 255, 0, 3.0, 0.0, 1.0),
            Vertex::new_with_rgba(CHUNK_SIZE_F, CHUNK_SIZE_F, 0.0, 255, 0, 0, 255, 0, 3.0, 0.0, 1.0),
            Vertex::new_with_rgba(0.0, CHUNK_SIZE_F, CHUNK_SIZE_F, 255, 0, 0, 255, 0, 3.0, 0.0, 1.0),
            Vertex::new_with_rgba(CHUNK_SIZE_F, CHUNK_SIZE_F, CHUNK_SIZE_F, 255, 0, 0, 255, 0, 3.0, 0.0, 1.0),
            Vertex::new_with_rgba(0.0, 0.0, 0.0, 0, 0, 255, 255, 0, 3.0, 0.0, 1.0),
            Vertex::new_with_rgba(0.0, 0.0, CHUNK_SIZE_F, 0, 0, 255, 255, 0, 3.0, 0.0, 1.0),
            Vertex::new_with_rgba(CHUNK_SIZE_F, 0.0, 0.0, 0, 0, 255, 255, 0, 3.0, 0.0, 1.0),
            Vertex::new_with_rgba(CHUNK_SIZE_F, 0.0, CHUNK_SIZE_F, 0, 0, 255, 255, 0, 3.0, 0.0, 1.0),
            Vertex::new_with_rgba(0.0, CHUNK_SIZE_F, 0.0, 0, 0, 255, 255, 0, 3.0, 0.0, 1.0),
            Vertex::new_with_rgba(0.0, CHUNK_SIZE_F, CHUNK_SIZE_F, 0, 0, 255, 255, 0, 3.0, 0.0, 1.0),
            Vertex::new_with_rgba(CHUNK_SIZE_F, CHUNK_SIZE_F, 0.0, 0, 0, 255, 255, 0, 3.0, 0.0, 1.0),
            Vertex::new_with_rgba(CHUNK_SIZE_F, CHUNK_SIZE_F, CHUNK_SIZE_F, 0, 0, 255, 255, 0, 3.0, 0.0, 1.0),
        ];

        if !self.is_surface_configured {
            return;
        }

        let surface = &self.gpu_context.surface;
        let device = &self.gpu_context.resources.device();
        let queue = &self.gpu_context.resources.queue();

        self.render_manager.update_indirect_buffer(device, queue);

        let mut encoder = self
            .frame_encoder
            .take()
            .unwrap_or(device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            }));

        if let Some(view_proj) = camera.view_proj().change() {
            queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(view_proj));
        }

        if let Some(cw) = camera.cw().change() {
            let chunk_borders_vertices: Vec<Vertex> = chunk_borders
                .iter()
                .map(|v| v.copy_with_pos(v.position[0] + cw[0], v.position[1] + cw[1], v.position[2] + cw[2]))
                .collect();

            queue.write_buffer(&self.chunk_borders_buffer, 0, bytemuck::cast_slice(&chunk_borders_vertices));
        };

        let output = surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.125,
                            g: 0.5,
                            b: 0.75,
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
                render_pass.set_pipeline(self.pipelines.opaque());
            } else {
                render_pass.set_pipeline(self.pipelines.opaque());
            }

            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            let meshes = self.render_manager.get_meshes_to_render();

            let mut _rendered_mesh_count = meshes.len();

            let _start = Instant::now();

            // World & Player meshes (other than local player)
            if _rendered_mesh_count > 0 {
                render_pass.set_vertex_buffer(0, self.render_manager.mesh_manager.get_buffer().slice(..));

                const CAN_MULTIDRAW: bool = true; // TODO: detect if the device supports multi-draw indirect

                if CAN_MULTIDRAW {
                    render_pass.multi_draw_indirect(&self.render_manager.indirect_buffer.buffer(), 0, meshes.len() as u32);
                } else {
                    const CMD_SIZE: u64 = std::mem::size_of::<DrawIndirectArgs>() as u64;
                    for i in 0.._rendered_mesh_count as u32 {
                        render_pass.draw_indirect(&self.render_manager.indirect_buffer.buffer(), i as u64 * CMD_SIZE);
                    }
                }
            }

            // Debug
            if self.wireframe || self.show_chunk_borders {
                render_pass.set_pipeline(&self.gizmo_render_pipeline);
                if self.wireframe {
                    render_pass.set_vertex_buffer(0, self.gizmo_buffer.slice(..));
                    render_pass.draw(0..6, 0..1);
                }
                if self.show_chunk_borders {
                    render_pass.set_vertex_buffer(0, self.chunk_borders_buffer.slice(..));
                    render_pass.draw(0..chunk_borders.len() as u32, 0..1);
                }
            }
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

        self.render_manager.mesh_manager.process_pending_destructions();

        self.frame_encoder = Some(device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Frame encoder"),
        }));

        self.render_manager.clear_render_queue();
    }

    pub fn dispose(&mut self) {
        self.is_surface_configured = false;

        self.camera_buffer.destroy();
        self.chunk_borders_buffer.destroy();
        self.depth_texture.destroy();
        // TODO: faire fonctionner -> self.diffuse_texture_array.dispose();
        self.gizmo_buffer.destroy();
        // TODO: faire fonctionner -> self.gpu_context.dispose();
        // TODO: faire fonctionner -> self.render_manager.dispose();
    }
}
