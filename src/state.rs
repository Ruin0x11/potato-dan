use std::collections::{HashMap, HashSet};

use GameContext;
use ai::{self, Ai, AiKind};
use calx_ecs::Entity;
use debug;
use ecs::Loadout;
use ecs::components::*;
use ecs::prefab;
use ecs::traits::*;
use engine::keys::KeyCode;
use point::*;
use point;
use rand::{self, Rng};
use renderer;
use util;
use world::{World, Event};

pub struct GameState {
    pub frame: u64,
    pub world: World,
}

impl GameState {
    pub fn new() -> Self {
        let mut world = World::new();

        for i in 0..30 {
            world.spawn(prefab::wall(), Point::new(3.0 + (i as f32 * 1.0), 0.0, 4.0));
            let x = rand::thread_rng().gen_range(0.0, 50.0);
            let z = rand::thread_rng().gen_range(0.0, 50.0);
            world.spawn(prefab::mob("Dood").c(Ai::new(AiKind::Guard)), Point::new(x, 0.0, z));
        }

        GameState {
            frame: 0,
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
    else {
        commands.push(Command::Wait);
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
    let recheck = context.state.frame % 60 == 0;

    update_camera(context);
    context.state.world.update_physics(recheck);

    step_ai(&mut context.state.world, recheck);
    step_physics(&mut context.state.world);
    step_holds(&mut context.state.world);
    step_gun(&mut context.state.world);
    step_bullet(&mut context.state.world);
    step_healths(&mut context.state.world);

    // TODO: move here
    context.state.world.handle_events();
    context.state.world.purge_dead();

    context.state.frame += 1;
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

fn step_holds(world: &mut World) {
    let mut holds = Vec::new();
    for entity in world.entities() {
        if world.ecs().holds.has(*entity) {
            holds.push(*entity);
        }
    }

    for hold in holds {
        let mut remove = HashSet::new();
        let mut adj = HashSet::new();
        let (pos, dir) = {
            let pos = world.ecs().positions.get_or_err(hold);
            (pos.pos, pos.dir)
        };

        {
            let holding = world.ecs().holds.get_or_err(hold);
            for entity in holding.0.keys() {
                if !world.contains(*entity) {
                    remove.insert(*entity);
                } else {
                    let is_holding = holding.0.get(entity).unwrap();
                    if !is_holding {
                        adj.insert(*entity);
                    }
                }
            }
        }

        for entity in remove {
            let mut holding = world.ecs_mut().holds.get_mut_or_err(hold);
            holding.0.remove(&entity);
        }

        for entity in adj {
            if let Some(to_set) = world.ecs_mut().positions.get_mut(hold) {
                to_set.pos = pos;
                to_set.dir = dir;
            }
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

    for gun in guns {
        let (holder_pos, holder_dir) = {
            let hold = world.ecs().holds.get(gun).unwrap();
            if hold.0.is_empty() {
                continue;
            }

            assert!(hold.0.keys().len() == 1);
            let owner = hold.0.keys().next().unwrap();

            let pos = world.ecs().positions.get_or_err(*owner);
            (pos.pos, pos.dir)
        };
        let mut gun_pos = world.ecs_mut().positions.get_mut_or_err(gun);
        gun_pos.dir = holder_dir;
        let p2: &mut Point = &mut gun_pos.pos;
        let p: Point = Point::new(holder_pos.x, holder_pos.y, holder_pos.z);
        p2.x = p.x;
        p2.y = p.y;
        p2.z = p.z;
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

fn step_ai(world: &mut World, recheck: bool) {
    let mut ais = Vec::new();
    for entity in world.entities() {
        if world.ecs().ais.has(*entity) {
            ais.push(*entity);
        }
    }

    for entity in ais {
        let action = ai::run(entity, world, recheck);
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
    let player = context.state.world.player().unwrap();
    match command {
        Command::Move(dir) => move_in_dir(&mut context.state.world, player, dir),
        Command::Jump => jump(&mut context.state.world, player),
        Command::Shoot => shoot(&mut context.state.world, player),
        Command::Wait => stop_moving(&mut context.state.world, player),

        Command::ReloadShaders => renderer::with_mut(|rc| rc.reload_shaders()),
        Command::Restart => restart_game(context),
        Command::Quit => (),
    }
}

fn move_in_dir(world: &mut World, entity: Entity, dir: Direction) {
    let mut phys = world.ecs_mut().physics.get_mut_or_err(entity);

    let offset = dir.to_movement_offset();
    phys.accel_x = offset.0 as f32 * 0.05;
    phys.accel_z = offset.1 as f32 * 0.05;
    phys.movement_frames += 1;
}

fn jump(world: &mut World, entity: Entity) {
    let mut phys = world.ecs_mut().physics.get_mut_or_err(entity);

    if phys.accel_y > -0.1 {
        phys.dy = -3.0
    }
}

fn stop_moving(world: &mut World, entity: Entity) {
    let mut phys = world.ecs_mut().physics.get_mut_or_err(entity);

    phys.movement_frames = 0;
    phys.accel_x = 0.0;
    phys.accel_z = 0.0;
}

fn shoot(world: &mut World, firing: Entity) {
    let gun = {
        let holds = world.ecs().holds.get_or_err(firing);
        let mut gun = None;
        for entity in holds.0.keys() {
            if let Some(g) = world.ecs().guns.get(*entity) {
                gun = Some(*g);
                break;
            }
        }
        gun
    };

    if let Some(gun) = gun {
        let pos = {
            let pos = world.ecs().positions.get_or_err(firing);
            let dir = pos.dir;
            point::relative(pos.pos, Point::new(1.5, 0.0, 0.0), pos.dir)
        };
        let dir = world.ecs().positions.get_or_err(firing).dir + rand::thread_rng().gen_range(-gun.spread, gun.spread);
        let bullet = world.spawn(prefab::bullet(firing), pos);
        let mut phys = world.ecs_mut().physics.get_mut_or_err(bullet);

        let dx = dir.cos();
        let dz = dir.sin();
        phys.impulse(Point::new(dx, 0.0, dz));
    }
}
