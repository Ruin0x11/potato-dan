use std::collections::{HashMap, HashSet};

use GameContext;
use ai::{self, Ai, AiKind, Action};
use calx_ecs::Entity;
use debug;
use engine::MouseState;
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
use world::{self, World, Event};

pub struct GameState {
    pub frame: u64,
    pub world: World,
}

impl GameState {
    pub fn new() -> Self {
        let siz = debug::get("world_size") as u32;
        let w = siz;
        let h = siz;
        let mut world = World::new(w, h);

        for i in 0..debug::get("charas") as u32 {
            let x = rand::thread_rng().gen_range(1.0, (w - 1) as f32);
            let z = rand::thread_rng().gen_range(1.0, (h - 1) as f32);
            let mob = world.spawn(prefab::mob("Dood").c(Ai::new(AiKind::SeekTarget)), Point::new(x, 0.0, z)).unwrap();
            let gun = world.spawn(prefab::gun(), point::zero()).unwrap();
            world.equip(mob, gun);
        }

        world::gen::city(&mut world);

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
    Bom,
    RotateCamera(f32),
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

    let z = input.get(&KeyCode::Z).map_or(false, |b| *b);
    if z {
        commands.push(Command::Restart);
    }

    let b = input.get(&KeyCode::B).map_or(false, |b| *b);
    if b {
        commands.push(Command::Bom);
    }

    let q = input.get(&KeyCode::Q).map_or(false, |b| *b);
    let e = input.get(&KeyCode::E).map_or(false, |b| *b);
    if q {
        commands.push(Command::RotateCamera(-0.1));
    }
    else if e {
        commands.push(Command::RotateCamera(0.1));
    }

    commands
}

pub fn game_step(context: &mut GameContext, input: &HashMap<KeyCode, bool>, mouse: &MouseState,
                 delta: f32) {
    let player_alive = context.state.world.player().map_or(false, |p| context.state.world.contains(p));
    if !player_alive {
        restart_game(context);
        return;
    }

    for command in get_commands(input) {
        run_command(context, command, delta);
    }

    update_look(context, mouse);

    process(context, delta);
}

fn process(context: &mut GameContext, delta: f32) {
    let poll = debug::get("poll") as u64;
    let recheck = context.state.frame % poll == 0;

    update_camera(context);

    step_ai(&mut context.state.world, true, delta);
    step_bomb(&mut context.state.world, delta);
    context.state.world.update_physics(recheck);
    step_physics(&mut context.state.world, delta);
    step_holds(&mut context.state.world);
    step_gun(&mut context.state.world);
    step_bullet(&mut context.state.world, delta);
    step_healths(&mut context.state.world);

    // TODO: move here
    context.state.world.handle_events();
    context.state.world.purge_dead();

    context.state.frame += 1;
}

fn step_physics(world: &mut World, delta: f32) {
    let mut objects = Vec::new();
    for entity in world.entities() {
        if world.ecs().physics.has(*entity) {
            objects.push(*entity);
        }
    }

    let mut groups = CollisionGroups::new();

    let friction = debug::get("friction");

    for entity in objects {
        let kind = world.ecs_mut().physics.get_mut_or_err(entity).kind;
        let mut dx;
        let mut dy;
        let mut dz;
        match kind {
            PhysicsKind::Physical => {
                let mut set_to_ground = false;
                {
                    let pos = world.ecs().positions.get_or_err(entity).pos;
                    let mut phys = world.ecs_mut().physics.get_mut_or_err(entity);

                    let top_speed = debug::get("top_speed");

                    let decel = 1.0 / (1.0 + (delta * friction));
                    phys.vel += phys.accel * delta;
                    phys.vel *= decel;

                    dx = phys.vel.x;
                    dy = phys.vel.y;
                    dz = phys.vel.z;

                    let on_ground = (pos.y + phys.vel.y * delta) > 0.001 && dy > -0.1;
                    phys.grounded = on_ground;

                    if on_ground {
                        set_to_ground = true;
                    } else if phys.grounded {
                        dy = 0.0;
                    }
                }

                let mut pos = world.ecs_mut().positions.get_mut_or_err(entity);
                pos.pos.x += dx * delta;
                if set_to_ground {
                    pos.pos.y = 0.0;
                } else {
                    pos.pos.y += dy * delta;
                }
                pos.pos.z += dz * delta;
            },
            PhysicsKind::Bullet => {
                {
                    let pos = world.ecs().positions.get_or_err(entity).pos;
                    let mut phys = world.ecs_mut().physics.get_mut_or_err(entity);
                    dx = phys.vel.x;
                    dy = phys.vel.y;
                    dz = phys.vel.z;
                }

                let mut pos = world.ecs_mut().positions.get_mut_or_err(entity);
                pos.pos.x += dx * delta;
                pos.pos.y += dy * delta;
                pos.pos.z += dz * delta;
            }
        }

        let (w, h) = world.size();
        let mut pos = world.ecs_mut().positions.get_mut_or_err(entity);
        pos.pos.x = util::clamp(pos.pos.x, 0.0, w as f32);
        pos.pos.z = util::clamp(pos.pos.z, 0.0, h as f32);
    }
}

fn step_bullet(world: &mut World, delta: f32) {
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
            bullet_compo.time_left -= delta;
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

fn step_bomb(world: &mut World, delta: f32) {
    let mut bombs = Vec::new();
    for entity in world.entities() {
        if world.ecs().bombs.has(*entity) {
            bombs.push(*entity);
        }
    }

    for bomb_ent in bombs {
        let exploded = world.ecs().bombs.get_or_err(bomb_ent).time_left < 0.0;
        if exploded {
            let pos = world.position(bomb_ent).unwrap().pos;
            explod(world, pos);
            world.push_event(Event::Destroy, bomb_ent)
        } else {
            let mut bomb = world.ecs_mut().bombs.get_mut_or_err(bomb_ent);
            bomb.time_left -= delta;
        }
    }
}

use std::f32::consts::PI;
use ncollide::world::{CollisionGroups, CollisionObject3, CollisionWorld, GeometricQueryType};
use nalgebra::{self, Isometry3, Point3, Translation3, Vector3, Matrix3x1};
use ncollide::query::Ray3;
use world::CollisionDataExtra;

fn explod(world: &mut World, point: Point) {
    let num_rays = 32;
    let groups = CollisionGroups::new();
    let mut impulses = Vec::new();

    let radius = debug::get("explod_size");
    let force = debug::get("explod_force");
    for i in 0u32..num_rays {
        let angle = (i as f32 / num_rays as f32) * PI * 2.0;
        let dir = Vector3::new(angle.sin() * radius, 0.0, angle.cos() * radius);
        let ray = Ray3::new(point, dir);
        for (obj, colray) in world.collision_world.interferences_with_ray(&ray, &groups) {
            if let CollisionDataExtra::Entity(entity) = *obj.data() {
                if world.ecs().physics.has(entity) {
                    let contact = point + dir * colray.toi;
                    if let Some(impulse) = blast_impulse(&point, contact, force / num_rays as f32) {
                        impulses.push((entity, impulse));
                    }
                }
            }
        }
    }

    for (entity, impulse) in impulses {
        let mut phys = world.ecs_mut().physics.get_mut_or_err(entity);
        phys.vel += impulse;
    }
}

fn blast_impulse(blast: &Point, center: Point, power: f32) -> Option<Vector3<f32>> {
    let dir = Vector3::from(center - blast);
    let dist = dir.norm();

    // ignore bodies exactly at the blast point - blast direction is undefined
    if (dist == 0.0) {
        return None;
    }

    let dist_inv = 1.0 / dist;
    let magni = power * dist_inv;
    Some(Vector3::from(dir * magni))
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
        {
            let mut gun = world.ecs_mut().guns.get_mut_or_err(gun);
            if !gun.shooting {
                gun.reset_refire();
            }
            gun.shooting = false;
        }
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

fn step_ai(world: &mut World, recheck: bool, delta: f32) {
    let mut ais = Vec::new();
    for entity in world.entities() {
        if world.ecs().ais.has(*entity) {
            ais.push(*entity);
        }
    }

    for entity in ais {
        stop_moving(world, entity);
        let action = ai::run(entity, world, recheck);
        match action {
            Some(Action::Go(dir)) => move_in_dir(world, entity, dir),
            Some(Action::Shoot(dir)) => {
                face_dir(world, entity, dir);
                shoot(world, entity, delta);
            }
            _ => stop_moving(world, entity),
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

fn update_look(context: &mut GameContext, mouse: &MouseState) {
    let size = renderer::with(|rc| rc.viewport.scaled_size());
    let center = ((size.0 / 2) as i32, (size.1 / 2) as i32);
    let mouse = (mouse.pos.0.max(0), mouse.pos.1.max(0));

    let theta = point::angle(center, mouse);

    let player = context.state.world.player().unwrap();
    let camera_rot = context.state.world.camera_rot();
    face_dir(&mut context.state.world, player, theta - camera_rot);
}

fn run_command(context: &mut GameContext, command: Command, delta: f32) {
    let player = context.state.world.player().unwrap();
    match command {
        Command::Move(dir) => move_in_dir(&mut context.state.world, player, dir),
        Command::Jump => jump(&mut context.state.world, player),
        Command::Shoot => shoot(&mut context.state.world, player, delta),
        Command::Wait => stop_moving(&mut context.state.world, player),
        Command::Bom => bom(&mut context.state.world, player),

        Command::RotateCamera(rot) => rotate_camera(&mut context.state.world, rot),
        Command::ReloadShaders => renderer::with_mut(|rc| rc.reload_shaders()),
        Command::Restart => restart_game(context),
        Command::Quit => (),
    }
}

fn rotate_camera(world: &mut World, rot: f32) {
    world.camera.map(|c| {
        let mut cam = world.ecs_mut().cameras.get_mut_or_err(c);
        cam.rot += rot;
    });
}

fn face_dir(world: &mut World, entity: Entity, dir: f32) {
    let mut pos = world.ecs_mut().positions.get_mut_or_err(entity);
    pos.dir = dir;
}

fn move_in_dir(world: &mut World, entity: Entity, dir: Direction) {
    let mut rot = dir.to_angle() + PI/2.0;

    if world.player().map_or(false, |p| p == entity) {
        let theta = world.camera_rot();
        rot += world.camera_rot();
    }

    let accel = debug::get("accel");
    let mut phys = world.ecs_mut().physics.get_mut_or_err(entity);
    phys.accel.x = rot.sin() as f32 * accel;
    phys.accel.z = rot.cos() as f32 * accel;
    phys.movement_frames += 1;
}

fn jump(world: &mut World, entity: Entity) {
    //let mut phys = world.ecs_mut().physics.get_mut_or_err(entity);

    //if phys.grounded {
    //    let jump = debug::get("jump");
    //    phys.dy = jump;
    //}
}

fn stop_moving(world: &mut World, entity: Entity) {
    let mut phys = world.ecs_mut().physics.get_mut_or_err(entity);

    phys.movement_frames = 0;
    phys.accel.x = 0.0;
    phys.accel.z = 0.0;
}

fn bom(world: &mut World, entity: Entity) {
    let pos = {
        let pos = world.ecs().positions.get_or_err(entity);
        let dir = pos.dir;
        point::relative(pos.pos, Point::new(1.5, 0.0, 0.0), pos.dir)
    };

    world.spawn(prefab::bomb(), pos);
}

fn shoot(world: &mut World, firing: Entity, delta: f32) {
    let gun = {
        let holds = world.ecs().holds.get_or_err(firing);
        let mut gun = None;
        for entity in holds.0.keys() {
            if let Some(g) = world.ecs().guns.get(*entity) {
                gun = Some(*entity);
                break;
            }
        }
        gun
    };

    if let Some(gun_ent) = gun {
        let bullet_count = {
            let mut gun = world.ecs_mut().guns.get_mut_or_err(gun_ent);
            gun.shoot(delta)
        };

        let pos = {
            let pos = world.ecs().positions.get_or_err(firing);
            let dir = pos.dir;
            point::relative(pos.pos, Point::new(1.5, 0.0, 0.0), pos.dir)
        };

        let spread = world.ecs().guns.get_or_err(gun_ent).spread;
        for count in 0..bullet_count {
            let dir = world.ecs().positions.get_or_err(firing).dir + rand::thread_rng().gen_range(-spread, spread);
            if let Some(bullet) = world.spawn(prefab::bullet(firing), pos) {
                let mut phys = world.ecs_mut().physics.get_mut_or_err(bullet);

                let speed = debug::get("bullet_speed");
                let dx = dir.cos() * speed;
                let dz = dir.sin() * speed;
                phys.vel += Vector::new(dx, 0.0, dz);
            }
        }
    }
}
