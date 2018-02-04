use std::collections::HashMap;

use GameContext;
use engine::keys::KeyCode;
use world::World;
use ecs::prefab;
use ecs::traits::*;
use point::*;
use calx_ecs::Entity;
use util;
use rand::{self, Rng};

pub struct GameState {
    pub world: World,
    pub player: Entity,
}

impl GameState {
    pub fn new() -> Self {
        let mut world = World::new();
        let player = world.spawn(prefab::mob("Dood"), Point::new(5.0, 5.0, 0.0));

        for i in 0..10 {
            world.spawn(prefab::wall(), Point::new(3.0 + (i as f32 * 1.0), 4.0, 0.0));
            let x = rand::thread_rng().gen_range(0.0, 10.0);
            let y = rand::thread_rng().gen_range(0.0, 10.0);
            world.spawn(prefab::mob("Dood"), Point::new(x, y, 0.0));
        }

        GameState {
            world: world,
            player: player,
        }
    }
}

/// A bindable command that can be executed by the player.
pub enum Command {
    Move(Direction),
    Wait,
    Quit,
}

pub fn get_command(input: &HashMap<KeyCode, bool>) -> Command {
    let h = input.get(&KeyCode::H).map_or(false, |b| *b);
    let j = input.get(&KeyCode::J).map_or(false, |b| *b);
    let k = input.get(&KeyCode::K).map_or(false, |b| *b);
    let l = input.get(&KeyCode::L).map_or(false, |b| *b);

    if h && j {
        return Command::Move(Direction::SW);
    }
    if h && k {
        return Command::Move(Direction::NW);
    }
    if l && j {
        return Command::Move(Direction::SE);
    }
    if l && k {
        return Command::Move(Direction::NE);
    }
    if h {
        return Command::Move(Direction::W);
    }
    if j {
        return Command::Move(Direction::S);
    }
    if k {
        return Command::Move(Direction::N);
    }
    if l {
        return Command::Move(Direction::E);
    }

    Command::Wait
}

pub fn game_step(context: &mut GameContext, input: &HashMap<KeyCode, bool>) {
    let command = get_command(input);
    run_command(context, command);

    process(context);
}

fn process(context: &mut GameContext) {
    let world = &mut context.state.world;
    let mut objects = Vec::new();
    for entity in world.entities() {
        if world.ecs().positions.has(*entity) {
            objects.push(*entity);
        }
    }

    for entity in objects {
        let mut dx = 0.0;
        let mut dy = 0.0;
        {
            let mut phys = world.ecs_mut().physics.get_mut_or_err(entity);
            phys.dx += phys.accel_x;
            phys.dy += phys.accel_y;
            phys.dx = util::clamp(phys.dx, -0.1, 0.1);
            phys.dy = util::clamp(phys.dy, -0.1, 0.1);
            dx = phys.dx;
            dy = phys.dy;
            phys.dx *= 0.8;
            phys.dy *= 0.8;

            if phys.dx.abs() < 0.01 {
                phys.dx = 0.0;
            }
            if phys.dy.abs() < 0.01 {
                phys.dy = 0.0;
            }
        }

        let mut pos = world.ecs_mut().positions.get_mut_or_err(entity);
        pos.x += dx;
        pos.y += dy;
    }
}

pub fn run_command(context: &mut GameContext, command: Command) {
    //let player = context.state.player;
    let mut charas = Vec::new();
    for entity in context.state.world.entities() {
        if context.state.world.ecs().charas.has(*entity) {
            charas.push(*entity);
        }
    }

    for entity in charas {
        let phys = context.state.world.ecs_mut().physics.get_mut_or_err(entity);

        match command {
            Command::Move(dir) => {
                let offset = dir.to_movement_offset();
                phys.direction = dir;
                phys.accel_x = offset.0 as f32 * 0.05;
                phys.accel_y = offset.1 as f32 * 0.05;
                phys.movement_frames += 1;
            },
            _ => {
                phys.movement_frames = 0;
                phys.accel_x = 0.0;
                phys.accel_y = 0.0;
            }
        }
    }
}
