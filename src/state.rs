use std::collections::HashMap;

use GameContext;
use engine::keys::KeyCode;
use world::World;
use ecs::prefab;
use ecs::Loadout;
use ecs::traits::*;
use ecs::components::*;
use point::*;
use calx_ecs::Entity;
use util;
use rand::{self, Rng};
use renderer;

pub struct GameState {
    pub world: World,
}

impl GameState {
    pub fn new() -> Self {
        let mut world = World::new();

        for i in 0..30 {
            world.spawn(prefab::wall(), Point::new(3.0 + (i as f32 * 1.0), 0.0, 4.0));
            let x = rand::thread_rng().gen_range(0.0, 50.0);
            let z = rand::thread_rng().gen_range(0.0, 50.0);
            world.spawn(prefab::mob("Dood"), Point::new(x, 0.0, z));
        }
        world.spawn(prefab::wall(), Point::new(3.0, 0.0, -4.0));
        world.spawn(prefab::mob("Dood"), Point::new(0.0, 0.0, 5.0));

        GameState {
            world: world,
        }
    }
}

/// A bindable command that can be executed by the player.
pub enum Command {
    Move(Direction),
    Jump,
    Wait,
    Quit,
    ReloadShaders,
    Restart,
}

pub fn get_commands(input: &HashMap<KeyCode, bool>) -> Vec<Command> {
    let mut commands = Vec::new();
    let h = input.get(&KeyCode::H).map_or(false, |b| *b);
    let j = input.get(&KeyCode::J).map_or(false, |b| *b);
    let k = input.get(&KeyCode::K).map_or(false, |b| *b);
    let l = input.get(&KeyCode::L).map_or(false, |b| *b);

    if h && j {
        commands.push(Command::Move(Direction::SW));
    }
    else if h && k {
        commands.push(Command::Move(Direction::NW));
    }
    else if l && j {
        commands.push(Command::Move(Direction::SE));
    }
    else if l && k {
        commands.push(Command::Move(Direction::NE));
    }
    else if h {
        commands.push(Command::Move(Direction::W));
    }
    else if j {
        commands.push(Command::Move(Direction::S));
    }
    else if k {
        commands.push(Command::Move(Direction::N));
    }
    else if l {
        commands.push(Command::Move(Direction::E));
    }

    let space = input.get(&KeyCode::Space).map_or(false, |b| *b);
    if space {
        commands.push(Command::Jump);
    }

    let r = input.get(&KeyCode::R).map_or(false, |b| *b);
    if r {
        commands.push(Command::ReloadShaders);
    }

    let q = input.get(&KeyCode::Q).map_or(false, |b| *b);
    if q {
        commands.push(Command::Restart);
    }

    if commands.is_empty() {
        commands.push(Command::Wait);
    }

    commands
}

pub fn game_step(context: &mut GameContext, input: &HashMap<KeyCode, bool>) {
    for command in get_commands(input) {
        run_command(context, command);
    }

    process(context);
}

fn process(context: &mut GameContext) {
    update_camera(context);
    context.state.world.update_physics();

    let world = &mut context.state.world;
    let mut objects = Vec::new();
    for entity in world.entities() {
        if world.ecs().physics.has(*entity) {
            objects.push(*entity);
        }
    }

    for entity in objects {
        let mut set_to_ground = false;
        let mut dx;
        let mut dy;
        let mut dz;
        {
            let pos = *world.ecs().positions.get_or_err(entity);
            let mut phys = world.ecs_mut().physics.get_mut_or_err(entity);
            phys.dx += phys.accel_x;
            phys.dy -= phys.accel_y;
            phys.dz += phys.accel_z;
            phys.dx = util::clamp(phys.dx, -0.1, 0.1);
            phys.dz = util::clamp(phys.dz, -0.1, 0.1);
            phys.dx *= 0.8;
            phys.accel_y -= 0.01;
            phys.dz *= 0.8;

            if phys.dx.abs() < 0.01 {
                phys.dx = 0.0;
            }
            if phys.dz.abs() < 0.01 {
                phys.dz = 0.0;
            }
            if (pos.y + phys.dy) > 0.01 {
                phys.dy = 0.0;
                phys.accel_y = 0.0;
                set_to_ground = true
            }

            dx = phys.dx;
            dy = phys.dy;
            dz = phys.dz;
        }

        let mut pos = world.ecs_mut().positions.get_mut_or_err(entity);
        pos.x += dx;
        if set_to_ground {
            pos.y = 0.0;
        } else {
            pos.y += dy;
        }
        pos.z += dz;
    }
}

fn update_camera(context: &mut GameContext) {
    let camera_entity = context.state.world.camera;

    if camera_entity.is_none() {
        return;
    }

    let camera_entity = camera_entity.unwrap();

    let following_pos = context.state.world.camera_pos();

    let mut pos = context.state.world.ecs_mut().positions.get_mut_or_err(camera_entity);
    if let Some(p) = following_pos {
        *pos = p;
    }
}

pub fn restart_game(context: &mut GameContext) {
    *context = GameContext::new();
}

pub fn run_command(context: &mut GameContext, command: Command) {
    match command {
        Command::Move(dir) => {
            let player = context.state.world.player().unwrap();
            let mut phys = context.state.world.ecs_mut().physics.get_mut_or_err(player);

            let offset = dir.to_movement_offset();
            phys.direction = dir;
            phys.accel_x = offset.0 as f32 * 0.05;
            phys.accel_z = offset.1 as f32 * 0.05;
            phys.movement_frames += 1;
        },
        Command::Jump => {
            let player = context.state.world.player().unwrap();
            let pos = *context.state.world.ecs().positions.get_or_err(player);
            let mut phys = context.state.world.ecs_mut().physics.get_mut_or_err(player);

            if phys.accel_y > -0.1 {
                phys.dy = -3.0
            }
        }
        Command::ReloadShaders => renderer::with_mut(|rc| rc.reload_shaders()),
        Command::Restart => restart_game(context),
        _ => {
            let player = context.state.world.player().unwrap();
            let mut phys = context.state.world.ecs_mut().physics.get_mut_or_err(player);

            phys.movement_frames = 0;
            phys.accel_x = 0.0;
            phys.accel_z = 0.0;
        }
    }
}
