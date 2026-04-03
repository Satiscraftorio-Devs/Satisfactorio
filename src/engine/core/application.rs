use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;
use winit::{application::ApplicationHandler, keyboard::KeyCode, keyboard::PhysicalKey};

use crate::engine::audio::GameAudioManager;
use crate::engine::core::state::State;
use crate::engine::render::render::{EngineFrameData, GameFrameData, RenderOptions, Renderer};
use winit::event::{DeviceEvent, DeviceId, KeyEvent, WindowEvent};
use winit::window::{CursorGrabMode, Window};

pub enum AppEvent {}

pub trait AppState {
    fn init(&mut self, renderer: &mut Renderer, audio_manager: &mut Option<GameAudioManager>);
    fn update(&mut self, frame: &EngineFrameData, render_options: &RenderOptions, data: &mut GameFrameData, renderer: &mut Renderer);
    fn fixed_update(&mut self, frame: &EngineFrameData, render_options: &RenderOptions, data: &mut GameFrameData);
    fn on_mouse_move(&mut self, dx: f64, dy: f64);
    fn on_key(&mut self, code: KeyCode, is_pressed: bool);
}

pub struct App<S: AppState> {
    engine_state: Option<State>,
    app_state: S,
    app_state_init: bool,
}

impl<S: AppState> App<S> {
    pub fn new(app_state: S) -> Self {
        Self {
            engine_state: None,
            app_state,
            app_state_init: false,
        }
    }
}

impl<S: AppState> ApplicationHandler<AppEvent> for App<S> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("resumed");
        let window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.engine_state = Some(pollster::block_on(State::new(window, &self.app_state)).unwrap());

        if !self.app_state_init {
            let state = self.engine_state.as_mut().unwrap();
            self.app_state.init(&mut state.renderer, &mut state.audio_manager);
            self.app_state_init = true;
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _: AppEvent) {
        println!("EVENT RECEIVED");
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        let Some(_state) = self.engine_state.as_mut() else {
            return;
        };

        if let DeviceEvent::MouseMotion { delta } = event {
            self.app_state.on_mouse_move(delta.0, delta.1);
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        let Some(state) = self.engine_state.as_mut() else {
            return;
        };

        state.update();
        state.game_frame_data.reset();

        let aspect = state.renderer.render_options.aspect;
        let znear = state.renderer.render_options.znear;
        let zfar = state.renderer.render_options.zfar;
        let render_options = RenderOptions { aspect, znear, zfar };

        self.app_state.update(
            &state.engine_frame_data,
            &render_options,
            &mut state.game_frame_data,
            &mut state.renderer,
        );

        for id in state.game_frame_data.visible_meshes.iter() {
            state.renderer.render_manager.mark_mesh_for_rendering(*id);
        }

        state.window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: winit::window::WindowId, event: WindowEvent) {
        let Some(state) = self.engine_state.as_mut() else {
            return;
        };

        match event {
            WindowEvent::Focused(true) => {
                state.window.set_cursor_visible(false);
                state.window.set_cursor_grab(CursorGrabMode::Confined).unwrap_or(());
            }
            WindowEvent::Focused(false) => {
                state.window.set_cursor_visible(true);
                state.window.set_cursor_grab(CursorGrabMode::None).unwrap_or(());
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => match state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                    let size: winit::dpi::PhysicalSize<u32> = state.window.inner_size();
                    state.resize(size.width, size.height);
                }
                Err(e) => {
                    log::error!("Unable to render {}", e);
                }
            },
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => {
                if code == KeyCode::Escape && key_state.is_pressed() {
                    event_loop.exit();
                    std::process::exit(0); // Brutal, à changer quand on aura fait l'asynchrone sur le greedy meshing
                } else if code == KeyCode::Digit1 && key_state.is_pressed() {
                    state.renderer.wireframe = !state.renderer.wireframe;
                    state.window.request_redraw();
                } else {
                    self.app_state.on_key(code, key_state.is_pressed());
                }
            }
            _ => {}
        }
    }
}
