use std::sync::Arc;

use roboscope_ipc::SimServices;
use winit::{application::ApplicationHandler, dpi::LogicalSize, event::{DeviceEvent, KeyEvent, WindowEvent}, event_loop::ActiveEventLoop, keyboard::PhysicalKey, window::Window};

use crate::app::state::State;

pub mod buffer;
pub mod camera;
pub mod egui;
pub mod gui;
pub mod input;
pub mod model;
pub mod pipeline;
pub mod resources;
pub mod state;
pub mod texture;
pub mod vertex;

pub struct App {
    sim_services: SimServices,
    state: Option<State>,
}

impl App {
    pub fn new() -> Self {
        let sim = SimServices::join(Some("Blobsim"), &roboscope_ipc::Config::default()).unwrap();
        Self {
            sim_services: sim,
            state: None
        }
    }
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes()
            .with_inner_size(LogicalSize::new(1280.0, 720.0));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let v5_input_handler = crate::app::input::V5InputHandler::new(&self.sim_services).unwrap();
        let disp_stream = self.sim_services.display_frames().unwrap().subscriber_builder().create().unwrap();
        self.state = Some(pollster::block_on(State::new(window, disp_stream, v5_input_handler)).unwrap());
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: State) {
        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        let egui_response = state.egui_state.state.on_window_event(&state.window, &event);
        if egui_response.consumed { return; }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                state.update();
                match state.render() {
                    Ok(_) => {},
                    Err(e) => {
                        log::error!("{e}");
                        event_loop.exit();
                    }
                }
            },
            WindowEvent::KeyboardInput { 
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(code),
                    state: key_state,
                    .. 
                }, ..
            } => state.handle_key(event_loop, code, key_state.is_pressed()),
            WindowEvent::MouseInput { state: button_state, button, .. } => {
                state.handle_mouse_input(button, button_state);
            },
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                state.handle_mouse_movement(dx, dy);
            }
            _ => {}
        }
    }
}