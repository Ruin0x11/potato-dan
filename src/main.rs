#![allow(warnings)]

#[macro_use]
extern crate calx_ecs;
#[macro_use]
extern crate macro_attr;
#[macro_use]
extern crate enum_derive;
//#[macro_use]
extern crate hlua;
//#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate glium;
#[macro_use]
extern crate imgui;

extern crate alga;
extern crate bincode;
extern crate cgmath;
extern crate crypto;
extern crate glob;
extern crate goap;
extern crate image;
extern crate imgui_glium_renderer;
extern crate nalgebra;
extern crate ncollide;
extern crate rand;
extern crate rusttype;
extern crate serde;
extern crate texture_packer;
extern crate toml;

#[macro_use]
mod macros;

mod ai;
mod debug;
mod ecs;
mod engine;
mod graphics;
mod point;
mod renderer;
mod state;
mod util;
mod world;

use std::collections::HashMap;
use glium::glutin::{self, VirtualKeyCode, ElementState};
use glium::glutin::WindowEvent::*;
use glium::glutin::ElementState::Pressed;
use glium::glutin::{Event, MouseButton, MouseScrollDelta, TouchPhase};
use state::GameState;
use engine::keys::KeyCode;
use engine::MouseState;

pub struct GameContext {
    pub state: GameState,
}

impl GameContext {
    pub fn new() -> Self {
        GameContext {
            state: GameState::new(),
        }
    }
}

fn main() {
    game_loop();

    println!("Exited cleanly.");
}

fn game_loop() {
    let mut context = GameContext::new();
    let mut keys = HashMap::new();
    let mut mouse_state = MouseState::default();

    renderer::with_mut(|rc| rc.update(&context.state.world));

    'outer: loop {
        let mut resize = None;
        let mut quit = false;
        let mut delta = 0.0;
        renderer::with_mut(|rc| {
            delta = rc.delta();
            rc.poll_events(|event| match event {
                glium::glutin::Event::WindowEvent { event, .. } => {
                    match event {
                        Closed => quit = true,
                        Resized(w, h) => {
                            resize = Some((w, h));
                        },
                        _ => (),
                    }

                    match event {
                        KeyboardInput { input, .. } => {
                            if let ElementState::Pressed = input.state {
                                if let Some(code) = input.virtual_keycode {
                                    match code {
                                        VirtualKeyCode::Escape => quit = true,
                                        _ => {
                                            let key = KeyCode::from(code);
                                            keys.insert(key, true);
                                        },
                                    }
                                }
                            }
                            if let ElementState::Released = input.state {
                                if let Some(code) = input.virtual_keycode {
                                    match code {
                                        _ => {
                                            let key = KeyCode::from(code);
                                            keys.insert(key, false);
                                        },
                                    }
                                }
                            }
                        },
                        CursorMoved { position: (x, y), .. } => mouse_state.pos = (x as i32, y as i32),
                        MouseInput { state, button, .. } => {
                            match button {
                                MouseButton::Left => mouse_state.pressed.0 = state == Pressed,
                                MouseButton::Right => mouse_state.pressed.1 = state == Pressed,
                                MouseButton::Middle => mouse_state.pressed.2 = state == Pressed,
                                _ => {}
                            }
                        }
                        MouseWheel {
                            delta: MouseScrollDelta::LineDelta(_, y),
                            phase: TouchPhase::Moved,
                            ..
                        } |
                        MouseWheel {
                            delta: MouseScrollDelta::PixelDelta(_, y),
                            phase: TouchPhase::Moved,
                            ..
                        } => mouse_state.wheel = y,
                        _ => (),
                    }
                },
                _ => (),
            });

            false
        });

        if quit {
            break 'outer;
        }

        if let Some((w, h)) = resize {
            renderer::with_mut(|renderer| {
                renderer.set_viewport(w, h);
                renderer.set_mouse(&mut mouse_state);
                renderer.update(&context.state.world);
                renderer.render();
            });
        }

        // Ensure that the renderer isn't borrowed during the game step, so it can be used in
        // the middle of any game routine (like querying the player for input)
        state::game_step(&mut context, &keys, &mouse_state, delta);

        renderer::with_mut(|renderer| renderer.update(&context.state.world));

        renderer::with_mut(|renderer| {
            if let Some((w, h)) = resize {
                renderer.set_viewport(w, h)
            }

            renderer.set_mouse(&mut mouse_state);
            renderer.render();
            renderer.step_frame();
        });
    }
}
