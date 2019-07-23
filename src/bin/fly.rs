#![allow(unused_imports)]
use glium::glutin::{
    ContextBuilder, DeviceEvent, ElementState, Event, EventsLoop, MouseButton, Window,
    WindowBuilder, WindowEvent, VirtualKeyCode,
};
use glium::{Display, Surface};
use std::env;
use std::fs::File;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
use std::collections::HashMap;
use ztar_rod::mod_dir::ModDir;
use ztar_rod::render::{Camera, Map, Renderer, Scene};

static KEY_CAM_CONTROL: VirtualKeyCode = VirtualKeyCode::LShift;

#[derive(Default)]
struct State {
    camera: Camera,
    closed: bool,
    is_clean: bool,
    keys_down: HashMap<VirtualKeyCode, bool>,
    //mouse_down: bool,
}

fn main() {
    let map_name = env::args()
        .nth(1)
        .unwrap_or_else(|| panic!("run as cargo --bin fly <map name>"));
    let mod_dir = ModDir::open(Path::new("./mod"));

    let mut events_loop = EventsLoop::new();
    let wb = WindowBuilder::new()
        .with_dimensions((1024, 768).into())
        .with_title("Map Viewer");
    let cb = ContextBuilder::new();
    let display = Display::new(wb, cb, &events_loop).unwrap();

    let renderer = Renderer::new(&display);
    let scene = Scene::new(&display, &mod_dir, &map_name).unwrap();
    let mut state = State::default();

    while !state.closed {
        if !state.is_clean {
            let mut target = display.draw();
            target.clear_color_and_depth((0.0, 0.0, 0.5, 1.0), 1.0);
            renderer.render(&mut target, &scene, &state.camera);
            target.finish().unwrap();

            state.is_clean = true;
        }

        sleep(Duration::from_millis(16));

        events_loop.poll_events(|ev| state.handle(&display, &ev));

        state.tick();
    }
}

use std::ops::Deref;

impl State {
    fn handle(&mut self, display: &glium::backend::glutin::Display, ev: &Event) {
        match ev {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => self.closed = true,
                WindowEvent::Refresh => self.is_clean = false,

                WindowEvent::KeyboardInput { input, .. } => if let Some(kc) = input.virtual_keycode {
                    match input.state {
                        ElementState::Pressed => self.keys_down.insert(kc, true),
                        ElementState::Released => self.keys_down.insert(kc, false),
                    };
                },

                /*
                WindowEvent::MouseInput {
                    state,
                    button: MouseButton::Right,
                    ..
                } => {
                    self.mouse_down = match state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    };
                },
                */

                _ => (),
            },

            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                if self.is_key_down(&KEY_CAM_CONTROL) {
                    self.is_clean = false;
                    self.camera.pan(cgmath::Deg(delta.0 as f32 * -0.1).into());
                    self.camera.tilt(cgmath::Deg(delta.1 as f32 * -0.1).into());
                }
            }

            _ => (),
        }
    }

    fn tick(&mut self) {
        if self.is_key_down(&KEY_CAM_CONTROL) {
            if self.is_key_down(&VirtualKeyCode::W) {
                self.camera.dolly(-10.0);
                self.is_clean = false;
            }

            if self.is_key_down(&VirtualKeyCode::S) {
                self.camera.dolly(10.0);
                self.is_clean = false;
            }

            if self.is_key_down(&VirtualKeyCode::A) {
                self.camera.truck(-10.0);
                self.is_clean = false;
            }

            if self.is_key_down(&VirtualKeyCode::D) {
                self.camera.truck(10.0);
                self.is_clean = false;
            }
        }
    }

    fn is_key_down(&self, kc: &VirtualKeyCode) -> bool {
        match self.keys_down.get(kc) {
            Some(true) => true,
            _ => false,
        }
    }
}
