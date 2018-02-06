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

extern crate bincode;
extern crate cgmath;
extern crate crypto;
extern crate glob;
extern crate goap;
extern crate image;
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
use state::GameState;
use engine::keys::KeyCode;

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
    let mut mouse = (0, 0);

    renderer::with_mut(|rc| rc.update(&context.state.world));

    'outer: loop {
        let mut resize = None;
        let mut quit = false;
        renderer::with_mut(|rc| {
            rc.poll_events(|event| match event {
                glutin::Event::WindowEvent { event, .. } => {
                    match event {
                        glutin::WindowEvent::Closed => quit = true,
                        glutin::WindowEvent::Resized(w, h) => {
                            resize = Some((w, h));
                        },
                        _ => (),
                    }

                    match event {
                        glutin::WindowEvent::KeyboardInput { input, .. } => {
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
                        glutin::WindowEvent::CursorMoved { device_id, position, .. } => {
                            mouse = (position.0 as i32, position.1 as i32);
                        }
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
                renderer.update(&context.state.world);
                renderer.render();
            });
        }

        // Ensure that the renderer isn't borrowed during the game step, so it can be used in
        // the middle of any game routine (like querying the player for input)
        state::game_step(&mut context, &keys, &mouse);

        renderer::with_mut(|renderer| renderer.update(&context.state.world));

        renderer::with_mut(|renderer| {
            if let Some((w, h)) = resize {
                renderer.set_viewport(w, h)
            }

            renderer.render();
            renderer.step_frame();
        });
    }
}
