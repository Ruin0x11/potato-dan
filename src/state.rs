use std::collections::HashMap;

use GameContext;
use point;
use debug;
use engine::keys::KeyCode;
use world::{World, Event};
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
    Shoot,
    Wait,
    Quit,
    ReloadShaders,
    Restart,
}

pub fn get_commands(input: &HashMap<KeyCode, bool>) -> Vec<Command> {
    let mut commands = Vec::new();
    let a = input.get(&KeyCode::A).map_or(false, |b| *b);
    let w = input.get(&KeyCode::W).map_or(false, |b| *b);
    let s = input.get(&KeyCode::S).map_or(false, |b| *b);
    let d = input.get(&KeyCode::D).map_or(false, |b| *b);

    if a && s {
        commands.push(Command::Move(Direction::SW));
    }
    else if a && w {
        commands.push(Command::Move(Direction::NW));
    }
    else if d && s {
        commands.push(Command::Move(Direction::SE));
    }
    else if d && w {
        commands.push(Command::Move(Direction::NE));
    }
    else if a {
        commands.push(Command::Move(Direction::W));
    }
    else if s {
        commands.push(Command::Move(Direction::S));
    }
    else if w {
        commands.push(Command::Move(Direction::N));
    }
    else if d {
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

    let f = input.get(&KeyCode::F).map_or(false, |b| *b);
    if f {
        commands.push(Command::Shoot);
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

pub fn game_step(context: &mut GameContext, input: &HashMap<KeyCode, bool>, mouse: &(i32, i32)) {
    let player_alive = context.state.world.player().map_or(false, |p| context.state.world.contains(p));
    if !player_alive {
        restart_game(context);
        return;
    }

    for command in get_commands(input) {
        run_command(context, command);
    }

    update_look(context, mouse);

    process(context);
}

fn process(context: &mut GameContext) {
    update_camera(context);
    context.state.world.update_physics();

    step_physics(&mut context.state.world);
    step_gun(&mut context.state.world);
    step_bullet(&mut context.state.world);
    step_healths(&mut context.state.world);
    // TODO: move here
    context.state.world.handle_events();
    context.state.world.purge_dead();
}

fn step_physics(world: &mut World) {
    let mut objects = Vec::new();
    for entity in world.entities() {
        if world.ecs().physics.has(*entity) {
            objects.push(*entity);
        }
    }

    for entity in objects {
        let kind = world.ecs_mut().physics.get_mut_or_err(entity).kind;
        match kind {
            PhysicsKind::Physical => {
                let mut set_to_ground = false;
                let mut dx;
                let mut dy;
                let mut dz;
                {
                    let pos = world.ecs().positions.get_or_err(entity).pos;
                    let mut phys = world.ecs_mut().physics.get_mut_or_err(entity);
                    phys.dx += phys.accel_x;
                    phys.dy -= phys.accel_y;
                    phys.dz += phys.accel_z;
                    phys.dx = util::clamp(phys.dx, -0.1, 0.1);
                    phys.dz = util::clamp(phys.dz, -0.1, 0.1);
                    phys.accel_y -= 0.01;

                    if phys.dx.abs() < 0.01 {
                        phys.dx = 0.0;
                    }
                    if phys.dz.abs() < 0.01 {
                        phys.dz = 0.0;
                    }
                    if (pos.y + phys.dy) > 0.01 {
                        phys.dy = 0.0;
                        phys.accel_y = 0.0;
                        set_to_ground = true;
                        phys.dx *= 0.85;
                        phys.dz *= 0.85;
                    }

                    dx = phys.dx;
                    dy = phys.dy;
                    dz = phys.dz;
                }

                let mut pos = world.ecs_mut().positions.get_mut_or_err(entity);
                pos.pos.x += dx;
                if set_to_ground {
                    pos.pos.y = 0.0;
                } else {
                    pos.pos.y += dy;
                }
                pos.pos.z += dz;
            },
            PhysicsKind::Bullet => {
                let mut dx;
                let mut dy;
                let mut dz;
                {
                    let pos = world.ecs().positions.get_or_err(entity).pos;
                    let mut phys = world.ecs_mut().physics.get_mut_or_err(entity);
                    dx = phys.dx;
                    dy = phys.dy;
                    dz = phys.dz;
                }

                let mut pos = world.ecs_mut().positions.get_mut_or_err(entity);
                pos.pos.x += dx;
                pos.pos.y += dy;
                pos.pos.z += dz;
            }
        }
    }
}

fn step_bullet(world: &mut World) {
    let mut bullets = Vec::new();
    for entity in world.entities() {
        if world.ecs().bullets.has(*entity) {
            bullets.push(*entity);
        }
    }

    for bullet in bullets {
        let mut remove = false;
        {
            let mut bullet_compo = world.ecs_mut().bullets.get_mut_or_err(bullet);
            bullet_compo.time_left -= 1.0;
            if bullet_compo.time_left < 0.0 {
                remove = true;
            }
        }

        if remove {
            world.push_event(Event::Destroy, bullet);
        }
    }
}

fn step_gun(world: &mut World) {
    let mut guns = Vec::new();
    for entity in world.entities() {
        if world.ecs().guns.has(*entity) {
            guns.push(*entity);
        }
    }

    // TODO: MAKE BETTER
    for gun in guns {
        {
            let active = {
                let gun_compo = world.ecs().guns.get_or_err(gun);
                if let Some(holder) = gun_compo.chara {
                    world.contains(holder)
                } else {
                    false
                }
            };

            if !active {
                let mut gun_compo = world.ecs_mut().guns.get_mut_or_err(gun);
                gun_compo.chara = None;
            }
        }

        let mut holder = None;
        let mut change = {
            let gun_compo = world.ecs().guns.get_or_err(gun);
            holder = gun_compo.chara;
            if let Some(h) = holder {
                world.contains(h)
            } else {
                false
            }
        };

        debug::add_text(format!("{:?} {}", holder, change));
        if holder.is_some() && change {
            let holder_dir = world.ecs().positions.get_or_err(holder.unwrap()).dir;
            let holder_pos = *world.ecs().positions.get_or_err(holder.unwrap()).pos;

            let mut gun_pos = world.ecs_mut().positions.get_mut_or_err(gun);
            gun_pos.dir = holder_dir;
            let p2: &mut Point = &mut gun_pos.pos;
            let p: Point = Point::new(holder_pos.x, holder_pos.y, holder_pos.z);
            p2.x = p.x;
            p2.y = p.y;
            p2.z = p.z;
        }
    }
}

fn step_healths(world: &mut World) {
    let mut healths = Vec::new();
    for entity in world.entities() {
        if world.ecs().healths.has(*entity) {
            healths.push(*entity);
        }
    }

    for health in healths {
        if world.ecs().healths.get_or_err(health).is_dead() {
            world.push_event(Event::Destroy, health);
        }
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
        let p2: &mut Point = &mut pos.pos;
        let p: Point = Point::new(p.x, p.y, p.z);
        p2.x = p.x;
        p2.y = p.y;
        p2.z = p.z;
    }
}

fn restart_game(context: &mut GameContext) {
    *context = GameContext::new();
}

fn update_look(context: &mut GameContext, mouse: &(i32, i32)) {
    let size = renderer::with(|rc| rc.viewport.scaled_size());
    let center = ((size.0 / 2) as i32, (size.1 / 2) as i32);
    let mouse = (mouse.0.max(0), mouse.1.max(0));

    let theta = point::angle(center, mouse);

    let player = context.state.world.player().unwrap();
    let mut pos = context.state.world.ecs_mut().positions.get_mut_or_err(player);
    pos.dir = theta;
}

fn run_command(context: &mut GameContext, command: Command) {
    match command {
        Command::Move(dir) => {
            let player = context.state.world.player().unwrap();

            {
                let mut phys = context.state.world.ecs_mut().physics.get_mut_or_err(player);

                let offset = dir.to_movement_offset();
                phys.accel_x = offset.0 as f32 * 0.05;
                phys.accel_z = offset.1 as f32 * 0.05;
                phys.movement_frames += 1;
            }
        },
        Command::Jump => {
            let player = context.state.world.player().unwrap();
            let mut phys = context.state.world.ecs_mut().physics.get_mut_or_err(player);

            if phys.accel_y > -0.1 {
                phys.dy = -3.0
            }
        }
        Command::ReloadShaders => renderer::with_mut(|rc| rc.reload_shaders()),
        Command::Shoot => {
            let player = context.state.world.player().unwrap();
            let kind = {
                let chara = context.state.world.ecs().charas.get_or_err(player);
                match chara.gun {
                    Some(gun) => {
                        let gun = context.state.world.ecs().guns.get_or_err(gun);
                        Some(gun.bullet)
                    },
                    None => None,
                }
            };

            if let Some(kind) = kind {
                let pos = context.state.world.ecs().positions.get_or_err(player).pos;
                let dir = context.state.world.ecs().positions.get_or_err(player).dir;
                let bullet = context.state.world.spawn(prefab::bullet(), pos);
                let mut phys = context.state.world.ecs_mut().physics.get_mut_or_err(bullet);

                let dx = dir.cos();
                let dz = dir.sin();
                phys.impulse(Point::new(dx, 0.0, dz));
            }
        }
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
