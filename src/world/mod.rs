use std::collections::HashMap;
use std::slice;

use calx_ecs::Entity;
use ecs::*;
use ecs::prefab;
use ecs::Loadout;
use ecs::traits::*;
use ecs::components::*;
use point;
use point::*;
use world::astar::Grid;

use ncollide::world::{CollisionGroups, CollisionObject3, CollisionWorld, GeometricQueryType};
use nalgebra::{self, Isometry3, Point3, Translation3, Vector3, Matrix3x1};
use ncollide::narrow_phase::{ContactAlgorithm3};
use ncollide::shape::{Ball, Cylinder, Cuboid, Plane, ShapeHandle3};
use ncollide::query::{self, Proximity};
use ncollide::events::{ContactEvents};

pub mod astar;

pub type CollideWorld = CollisionWorld<Point, Isometry3<f32>, CollisionDataExtra>;

#[derive(Clone, Copy, Debug)]
pub enum CollisionDataExtra {
    Entity(Entity),
    Node,
}

pub struct World {
    ecs: Ecs,
    player: Option<Entity>,
    pub camera: Option<Entity>,
    pub collision_world: CollideWorld,
    pub grid: Grid,
    shapes: HashMap<PhysicsShape, CollisionData>,
    events: Vec<(Event, Entity)>,
    kill_list: Vec<Entity>,

    // in 32 pixel increments
    size: (u32, u32),
}

#[derive(Clone)]
struct CollisionData {
    pub shape: ShapeHandle3<f32>,
    pub groups: CollisionGroups,
}

fn shape_handles() -> HashMap<PhysicsShape, CollisionData> {
    let mut map = HashMap::new();

    let mut groups = CollisionGroups::new();
    groups.set_membership(&[1]);
    groups.set_whitelist(&[1, 2, 3]);
    groups.set_blacklist(&[]);
    map.insert(PhysicsShape::Chara, CollisionData {
        shape: ShapeHandle3::new(Cylinder::new(0.5, 0.5)),
        groups: groups,
    });

    let mut groups = CollisionGroups::new();
    groups.set_membership(&[2]);
    groups.set_whitelist(&[1, 3, 4]);
    groups.set_blacklist(&[2]);
    map.insert(PhysicsShape::Wall, CollisionData {
        shape: ShapeHandle3::new(Cuboid::new(Vector3::new(0.5, 10.0, 0.5))),
        groups: groups,
    });


    let mut groups = CollisionGroups::new();
    groups.set_membership(&[3]);
    groups.set_whitelist(&[1, 2]);
    groups.set_blacklist(&[]);
    map.insert(PhysicsShape::Bullet, CollisionData {
        shape: ShapeHandle3::new(Ball::new(0.5)),
        groups: groups,
    });

    map
}

impl World {
    pub fn new() -> Self {
        let size = (64, 64);
        let mut collision_world = CollisionWorld::new(0.02);
        let grid = Grid::new(&mut collision_world, size);
        let mut world = World {
            ecs: Ecs::new(),
            player: None,
            camera: None,
            collision_world: collision_world,
            grid: grid,
            shapes: shape_handles(),
            events: Vec::new(),
            kill_list: Vec::new(),
            size: size,
        };

        let player = world.spawn(prefab::mob("Dood"), Point::new(0.0, 0.0, 0.0)).unwrap();
        let camera = world.spawn(Loadout::new().c(Camera::new(player)), point::zero()).unwrap();

        let gun = world.spawn(prefab::gun(), point::zero()).unwrap();
        world.equip(player, gun);

        world.player = Some(player);
        world.camera = Some(camera);
        world
    }

    // immut

    pub fn ecs(&self) -> &Ecs {
        &self.ecs
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub fn entities(&self) -> slice::Iter<Entity> {
        self.ecs.iter()
    }

    pub fn position(&self, entity: Entity) -> Option<&Position> {
        self.ecs.positions.get(entity)
    }

    pub fn player(&self) -> Option<Entity> {
        self.player
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.ecs.contains(entity)
    }

    pub fn camera_pos(&self) -> Option<Point> {
        self.camera.map(|c| self.ecs().cameras.get_or_err(c)).and_then(|cam| {
            if !self.contains(cam.following) || !self.ecs().positions.has(cam.following) {
                None
            } else {
                self.ecs().positions.get(cam.following).cloned().map(|p| p.pos)
            }
        })
    }

    pub fn in_bounds(&self, pos: &Point) -> bool {
        pos.x >= 0.0 && pos.z >= 0.0 && pos.x < self.size.0 as f32 && pos.z < self.size.1 as f32
    }

    // mut

    pub fn ecs_mut(&mut self) -> &mut Ecs {
        &mut self.ecs
    }

    pub fn spawn(&mut self, mut loadout: Loadout, pos: Point) -> Option<Entity> {
        if !self.in_bounds(&pos) {
            return None;
        }

        loadout = loadout.c(Position::new(pos)).c(Holds::new());

        let entity = loadout.make(&mut self.ecs);

        if self.ecs.physics.contains(entity) {
            let collision_data = {
                let phys = self.ecs.physics.get_or_err(entity);
                self.shapes.get(&phys.shape).cloned().unwrap()
            };
            let pos = self.ecs.positions.get_or_err(entity).clone();
            let obj_pos = Isometry3::new(Vector3::new(pos.pos.x, pos.pos.y, pos.pos.z), nalgebra::zero());
            let handle = self.collision_world.add(obj_pos,
                                                  collision_data.shape,
                                                  collision_data.groups,
                                                  GeometricQueryType::Contacts(0.0, 0.0),
                                                  CollisionDataExtra::Entity(entity));

            let mut phys = self.ecs.physics.get_mut_or_err(entity);
            phys.handle = Some(handle);
        }

        Some(entity)
    }

    pub fn remove(&mut self, entity: Entity) {
        if self.ecs.physics.contains(entity) {
            let handle = self.ecs.physics.get_mut_or_err(entity).handle;

            if let Some(handle) = handle {
                self.collision_world.remove(&[handle]);
            }
        }

        self.ecs.remove(entity);
    }

    pub fn kill(&mut self, entity: Entity) {
        self.ecs_mut().healths.map_mut(|h| h.kill(), entity);
    }

    pub fn purge_dead(&mut self) {
        while let Some(e) = self.kill_list.pop() {
            self.remove(e);
        }
    }

    pub fn equip(&mut self, chara: Entity, gun: Entity) {
        {
            let mut holds = self.ecs.holds.get_mut_or_err(chara);
            holds.0.insert(gun, true);
        }
        {
            let mut holds = self.ecs.holds.get_mut_or_err(gun);
            holds.0.insert(chara, false);
        }
    }

    pub fn update_physics(&mut self, remake_grid: bool) {
        self.update_world_to_physics();
        self.update_collision_world();
        self.update_physics_to_world(remake_grid);
    }

    fn update_world_to_physics(&mut self) {
        let mut entities = Vec::new();
        for entity in self.entities() {
            if self.ecs.physics.has(*entity) {
                entities.push(*entity);
            }
        }

        for entity in entities.iter() {
            {
                let pos = self.ecs.positions.get_or_err(*entity);
                let handle = self.ecs.physics.get_or_err(*entity).handle;
                if let Some(handle) = handle {
                    //println!("{:?}", pos.translation.vector);
                    if self.collision_world.collision_object(handle).is_none() {
                        // This should happen exactly once for each object when it is first created.
                        // `CreateObjectSys` has added the object, but the collision world has
                        // not been updated yet, so changing the position here would be an error.
                        continue;
                    }
                    let pos = Isometry3::new(Vector3::new(pos.pos.x, pos.pos.y, pos.pos.z), nalgebra::zero());
                    self.collision_world.set_position(handle, pos);
                }
            }
        }

    }

    fn update_collision_world(&mut self) {
        self.collision_world.update();
    }

    fn update_physics_to_world(&mut self, remake_grid: bool) {
        let mut vec = Vec::new();
        for (e1, e2, ca) in self.collision_world.contact_pairs() {
            let mut contacts = Vec::new();
            ca.contacts(&mut contacts);
            for contact in contacts {
                let move_vec = contact.normal.unwrap() * contact.depth * -0.5;
                let move_vec_b = move_vec * -1.0;
                vec.push((*e1.data(), *e2.data(), move_vec, move_vec_b));
            }
        }

        for (a, b, m1, m2) in vec {
            self.collide_two(a, b, &m1);
            self.collide_two(b, a, &m2);
        }

        // recalculating astar grid is expensive, only try once in a while.
        if remake_grid {
            self.discretize_grid();
        }
    }

    fn discretize_grid(&mut self) {
        self.grid.discretize(&self.collision_world);
    }

    fn collide_two(&mut self, a: CollisionDataExtra, b: CollisionDataExtra, move_vec: &Matrix3x1<f32>) {
        if let CollisionDataExtra::Entity(a) = a {
            if let CollisionDataExtra::Entity(b) = b {
                self.collide_two_entities(a, b, move_vec);
            }
        }
    }

    fn collide_two_entities(&mut self, a: Entity, b: Entity, move_vec: &Matrix3x1<f32>) {
        if !self.ecs.bullets.has(a) && !self.ecs.bullets.has(b) && self.ecs.charas.has(a) {
            let mut on_ground = false;
            if let Some(pos) = self.ecs.positions.get_mut(a) {
                log!("{:?}", move_vec);
                pos.pos.x += move_vec.x;
                pos.pos.y += move_vec.y;
                pos.pos.z += move_vec.z;

                on_ground = move_vec.y.abs() > 0.0;
            }
            if let Some(phys) = self.ecs.physics.get_mut(a) {
                if on_ground {
                    phys.dy = 0.0;
                    phys.accel_y = 0.0;
                }
            }
        }

        if self.ecs().bullets.has(a) {
            if self.ecs().charas.has(b) {
                // TODO: blacklist bullet from other team
                let fired_by = self.ecs().bullets.get_or_err(a).fired_by;
                if fired_by == b {
                    return;
                }

                let damage = self.ecs().bullets.get_or_err(a).damage;
                self.push_event(Event::Hurt(damage), b);
            }
            self.push_event(Event::Collide(move_vec.clone()), b);
            self.push_event(Event::Destroy, a);
        }
    }

    pub fn push_event(&mut self, event: Event, entity: Entity) {
        self.events.push((event, entity));
    }

    pub fn handle_events(&mut self) {
        while let Some((event, entity)) = self.events.pop() {
            match event {
                Event::Hurt(damage) => {
                    if let Some(health) = self.ecs_mut().healths.get_mut(entity) {
                        health.hurt(damage);
                    }
                },
                Event::Destroy => {
                    self.kill_list.push(entity);
                },
                Event::Collide(vec) => {
                    if let Some(phys) = self.ecs_mut().physics.get_mut(entity) {
                        phys.impulse(Vector3::new(-vec.x, 0.0, -vec.z));
                    }
                }
            }
        }
    }
}

pub enum Event {
    Hurt(i32),
    Destroy,
    Collide(Matrix3x1<f32>),
}
