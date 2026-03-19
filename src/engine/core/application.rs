use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;
use winit::{application::ApplicationHandler, keyboard::PhysicalKey};

use crate::engine::core::state::State;
use crate::engine::render::camera::Camera;
use crate::engine::render::render::Renderer;
use winit::event::{DeviceEvent, DeviceId, KeyEvent, WindowEvent};
use winit::window::{CursorGrabMode, Window};

pub enum AppEvent {

}

pub struct App {
    state: Option<State>,
    update: fn(f32, &mut Renderer),
}

impl App {
    pub fn new(
        update: fn(f32, &mut Renderer)
    ) -> Self {
        Self {
            state: None,
            update
        }
    }
}

impl ApplicationHandler<AppEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("resumed");
        let window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.state = Some(pollster::block_on(State::new(window)).unwrap());
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _: AppEvent) {
        println!("EVENT RECEIVED");
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let Some(state) = self.state.as_mut() else {
            return;
        };

        if let DeviceEvent::MouseMotion { delta } = event {
            state.inputs.set_mouse_delta(delta);
            // state.game_state.camera_controller.process_mouse(delta.0, delta.1);
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        let Some(state) = self.state.as_mut() else {
            return;
        };

        state.update();
        (self.update)(state.frame_data.dt, &mut state.renderer);
        state.window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = self.state.as_mut() else {
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
            WindowEvent::RedrawRequested => {
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size: winit::dpi::PhysicalSize<u32> = state.window.inner_size();
                        state.resize(size.width, size.height);
                    }
                    Err(e) => {
                        log::error!("Unable to render {}", e);
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => state.handle_key(event_loop, code, key_state.is_pressed()),
            _ => {}
        }
    }
}