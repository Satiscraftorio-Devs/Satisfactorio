use crate::common::geometry::vertex::Vertex;
use crate::engine::core::inputs::InputState;
use crate::engine::render::camera::RenderCamera;
use crate::engine::render::render::{EngineFrameData, GameFrameData, GpuContext, RenderManager, Renderer};
use crate::engine::render::text::TextRenderer;
use std::sync::Arc;
use std::time::Instant;
use wgpu::util::DeviceExt;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;

// This will store the state of our game
pub struct State {
    pub window: Arc<Window>,
    pub engine_frame_data: EngineFrameData,
    pub game_frame_data: GameFrameData,
    pub renderer: Renderer,
    text_renderer: TextRenderer,
    pub inputs: InputState,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
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
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);

        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
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

        let diffuse_bytes = include_bytes!("../../../assets/images/happy-tree.png");
        let diffuse_texture =
            crate::engine::render::texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
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
                    // 4.
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
                // cull_mode: None,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Line,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // pixels plus proches gagnent
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
                    // 4.
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
                // cull_mode: None,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // pixels plus proches gagnent
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
                    // 4.
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
                // cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // pixels plus proches gagnent
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
            Vertex::new_with_rgb(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0, 3.0),
            Vertex::new_with_rgb(1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0, 3.0),
            Vertex::new_with_rgb(0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0, 3.0),
            Vertex::new_with_rgb(0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0, 3.0),
            Vertex::new_with_rgb(0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0, 3.0),
            Vertex::new_with_rgb(0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0, 3.0),
        ];

        let gizmo_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gizmo Buffer"),
            contents: bytemuck::cast_slice(&gizmo),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Vertex Buffer"),
        //     contents: bytemuck::cast_slice(VERTICES),
        //     usage: wgpu::BufferUsages::VERTEX,
        // });

        // let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Index Buffer"),
        //     contents: bytemuck::cast_slice(INDICES),
        //     usage: wgpu::BufferUsages::INDEX,
        // });

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
            format: wgpu::TextureFormat::Depth24PlusStencil8, // ou Depth32Float
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
            diffuse_texture,
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

        let inputs = InputState::new();

        Ok(Self {
            window,
            engine_frame_data,
            game_frame_data,
            renderer,
            text_renderer,
            inputs,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
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
                    format: wgpu::TextureFormat::Depth24PlusStencil8, // ou Depth32Float
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

            // println!(
            //     "FPS: {} dt: {}s",
            //     self.engine_frame_data.fps, self.engine_frame_data.dt,
            // );
        }
    }

    pub fn update(&mut self) {
        self.frame_update();
        self.text_renderer.update_text(self.engine_frame_data.fps);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // self.window.request_redraw();

        // We can't render unless the surface is configured
        // if !self.renderer.is_surface_configured {
        //     return Ok(());
        // }

        // let output = self.surface.get_current_texture()?;

        // let view = output
        //     .texture
        //     .create_view(&wgpu::TextureViewDescriptor::default());

        // let mut encoder = self
        //     .device
        //     .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        //         label: Some("Render Encoder"),
        //     });

        // {
        //     let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        //         label: Some("Render Pass"),
        //         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        //             view: &view,
        //             resolve_target: None,
        //             depth_slice: None,
        //             ops: wgpu::Operations {
        //                 load: wgpu::LoadOp::Clear(wgpu::Color {
        //                     r: 0.9,
        //                     g: 0.9,
        //                     b: 0.9,
        //                     a: 1.0,
        //                 }),
        //                 store: wgpu::StoreOp::Store,
        //             },
        //         })],
        //         depth_stencil_attachment: None,
        //         occlusion_query_set: None,
        //         timestamp_writes: None,
        //         multiview_mask: None,
        //     });

        //     let render_context = RenderContext::new(
        //         &self.frame_data,
        //         &self.game_state,
        //         &self.renderer
        //     );

        //     render_world(&mut render_pass, &render_context);
        //     render_gizmo(&mut render_pass, &render_context);

        //     self.text_renderer.prepare(&self.device, &self.queue);
        //     self.text_renderer.render(&self.device, &self.queue, &mut render_pass);
        // }

        // // submit will accept anything that implements IntoIter
        // self.queue.submit(std::iter::once(encoder.finish()));
        // output.present();

        self.renderer.render(&self.game_frame_data.camera);

        Ok(())
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        if code == KeyCode::Escape && is_pressed {
            event_loop.exit();
        } else if code == KeyCode::Digit1 && is_pressed {
            self.renderer.wireframe = !self.renderer.wireframe;
            self.window.request_redraw();
        } else {
            self.inputs.set_key_press(code, is_pressed);
            // self.game_state.camera_controller.handle_key(code, is_pressed);
        }
    }
}
