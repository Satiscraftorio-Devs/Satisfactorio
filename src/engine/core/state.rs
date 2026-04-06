use std::sync::Arc;

use crate::common::geometry::vertex::Vertex;
use crate::engine::audio::GameAudioManager;
use crate::engine::render::camera::RenderCamera;
use crate::engine::render::render::{EngineFrameData, GameFrameData, GpuContext, RenderManager, RenderOptions, Renderer};
use crate::engine::render::text::TextRenderer;
use crate::engine::render::texture::TextureArrayManager;
use std::time::Instant;
use wgpu::util::DeviceExt;
use winit::window::Window;

pub struct State {
    pub window: Arc<Window>,
    pub engine_frame_data: EngineFrameData,
    pub game_frame_data: GameFrameData,
    pub renderer: Renderer,
    text_renderer: TextRenderer,
    pub audio_manager: Option<GameAudioManager>,
}

impl State {
    pub async fn new<S: crate::engine::core::application::AppState>(window: Arc<Window>, _app_state: &S) -> anyhow::Result<Self> {
        let audio_manager = GameAudioManager::new(&window);
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::POLYGON_MODE_LINE,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // let diffuse_bytes = include_bytes!("../../../assets/images/happy-tree.png");
        // let diffuse_texture = crate::engine::render::texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        macro_rules! load_textures {
            ($($name:ident: $path:literal),*) => {
                vec![$(
                    image::load_from_memory(include_bytes!($path))
                        .expect(concat!("Failed to load texture: ", $path))
                        .to_rgba8()
                ),*]
            };
        }

        let textures_data = load_textures!(
            grass: "../../../assets/images/grass.png",
            dirt: "../../../assets/images/dirt.png",
            stone: "../../../assets/images/stone.png"
        );

        let (width, height) = textures_data[0].dimensions();
        for (i, data) in textures_data.iter().enumerate() {
            assert_eq!(
                data.dimensions(),
                (width, height),
                "All textures must have same dimensions (texture {})",
                i
            );
        }

        let textures: Vec<&[u8]> = textures_data.iter().map(|d| d.as_ref()).collect();
        let texture_array = TextureArrayManager::make_array(&device, &queue, textures, width, height);

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_array.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_array.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let camera_uniform = RenderCamera::new();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../assets/shaders/shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
            immediate_size: 0,
        });

        let wireframe_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::buffer_layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Line,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: 1,
                    slope_scale: 1.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::buffer_layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: 1,
                    slope_scale: 1.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let gizmo_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::buffer_layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let gizmo = [
            Vertex::new_with_rgb(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0, 3.0, 0.0, 0.0),
            Vertex::new_with_rgb(1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0, 3.0, 0.0, 0.0),
            Vertex::new_with_rgb(0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0, 3.0, 0.0, 0.0),
            Vertex::new_with_rgb(0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0, 3.0, 0.0, 0.0),
            Vertex::new_with_rgb(0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0, 3.0, 0.0, 0.0),
            Vertex::new_with_rgb(0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0, 3.0, 0.0, 0.0),
        ];

        let gizmo_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gizmo Buffer"),
            contents: bytemuck::cast_slice(&gizmo),
            usage: wgpu::BufferUsages::VERTEX,
        });

        window
            .set_cursor_grab(winit::window::CursorGrabMode::Confined)
            .expect("Capture souris");
        window.set_cursor_visible(false);

        let engine_frame_data = EngineFrameData::new();
        let mut game_frame_data = GameFrameData::blank();

        game_frame_data.camera = camera_uniform;

        let depth_size = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };

        let depth_texture_desc = wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: depth_size,
            view_formats: &[wgpu::TextureFormat::Depth24PlusStencil8],
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        };

        let depth_texture = device.create_texture(&depth_texture_desc);

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let gpu_context = GpuContext {
            surface,
            device,
            queue,
            config,
        };

        let render_manager = RenderManager::new();

        let renderer = Renderer::new(
            false,
            wireframe_render_pipeline,
            render_pipeline,
            diffuse_bind_group,
            texture_array,
            camera_buffer,
            camera_bind_group,
            gizmo_render_pipeline,
            gizmo_buffer,
            (size.width, size.height),
            gpu_context,
            render_manager,
            depth_texture,
            depth_view,
        );

        let text_renderer = TextRenderer::new(
            &renderer.gpu_context.device,
            &renderer.gpu_context.queue,
            renderer.gpu_context.config.format,
        );

        Ok(Self {
            window,
            engine_frame_data,
            game_frame_data,
            renderer,
            text_renderer,
            audio_manager: Some(audio_manager.expect("Failed to load audio manager in state")),
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.renderer.render_options = RenderOptions {
                aspect: (width as f32) / (height as f32),
                znear: self.renderer.render_options.znear,
                zfar: self.renderer.render_options.zfar,
            };
            self.renderer.gpu_context.config.width = width;
            self.renderer.gpu_context.config.height = height;
            self.renderer
                .gpu_context
                .surface
                .configure(&self.renderer.gpu_context.device, &self.renderer.gpu_context.config);
            self.renderer.is_surface_configured = true;
            self.text_renderer.resize(width, height);
            self.renderer.depth_texture = self.renderer.gpu_context.device.create_texture(&wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: width,
                    height: height,
                    depth_or_array_layers: 1,
                },
                ..wgpu::TextureDescriptor {
                    label: Some("Depth Texture"),
                    view_formats: &[wgpu::TextureFormat::Depth24PlusStencil8],
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                    size: Default::default(),
                }
            });
            self.renderer.depth_view = self.renderer.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        }
    }

    pub fn frame_update(&mut self) {
        let now = Instant::now();
        let dt = now - self.engine_frame_data.last_frame;

        self.engine_frame_data.last_frame = now;
        self.engine_frame_data.dt = dt.as_secs_f32();

        self.engine_frame_data.frame_count += 1;
        self.engine_frame_data.fps_timer += dt.as_secs_f32();

        if self.engine_frame_data.fps_timer >= 1.0 {
            self.engine_frame_data.fps = self.engine_frame_data.frame_count;
            self.engine_frame_data.frame_count = 0;
            self.engine_frame_data.fps_timer = self.engine_frame_data.fps_timer - 1.0;
        }

        println!(
            "FPS (avg): {:4.0} FPS (last): {:4.0} dt: {:.3}ms",
            self.engine_frame_data.fps,
            1.0 / self.engine_frame_data.dt,
            self.engine_frame_data.dt * 1000.0
        );
    }

    pub fn update(&mut self) {
        self.frame_update();
        self.text_renderer.update_text(self.engine_frame_data.fps);
        if let Some(ref mut audio) = self.audio_manager {
            audio.update();
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.renderer.render(&self.game_frame_data.camera, Some(&mut self.text_renderer));

        Ok(())
    }
}
