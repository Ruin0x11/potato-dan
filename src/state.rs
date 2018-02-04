use std::collections::HashMap;

use GameContext;
use engine::keys::KeyCode;
use world::World;
use ecs::prefab;
use ecs::traits::*;
use point::*;
use calx_ecs::Entity;
use util;

pub struct GameState {
    pub world: World,
    pub player: Entity,
}

impl GameState {
    pub fn new() -> Self {
        let mut world = World::new();
        let player = world.spawn(prefab::mob("Dood"), Point::new(0.0, 0.0, 0.0));
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
    let mut charas = Vec::new();
    for entity in world.entities() {
        if world.ecs().positions.has(*entity) &&
            world.ecs().appearances.has(*entity) {
                charas.push(*entity);
            }
    }

    for entity in charas {
        let mut pos = world.ecs_mut().positions.get_mut_or_err(entity);
        pos.dx += pos.accel_x;
        pos.dy += pos.accel_y;
        pos.dx = util::clamp(pos.dx, -0.3, 0.3);
        pos.dy = util::clamp(pos.dy, -0.3, 0.3);
        pos.pos.x += pos.dx;
        pos.pos.y += pos.dy;
        pos.dx *= 0.8;
        pos.dy *= 0.8;

        if pos.dx.abs() < 0.01 {
            pos.dx = 0.0;
        }
        if pos.dy.abs() < 0.01 {
            pos.dy = 0.0;
        }
    }
}

pub fn run_command(context: &mut GameContext, command: Command) {
    let player = context.state.player;
    let pos = context.state.world.ecs_mut().positions.get_mut_or_err(player);
    match command {
        Command::Move(dir) => {
            let offset = dir.to_movement_offset();
            pos.direction = dir;
            pos.accel_x = offset.0 as f32 * 0.1;
            pos.accel_y = offset.1 as f32 * 0.1;
            pos.movement_frames += 1;
        },
        _ => {
            pos.movement_frames = 0;
            pos.accel_x = 0.0;
            pos.accel_y = 0.0;
        }
    }
}
