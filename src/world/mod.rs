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

use ncollide::world::{CollisionGroups, CollisionObject3, CollisionWorld, GeometricQueryType};
use nalgebra::{self, Isometry3, Point3, Translation3, Vector3};
use ncollide::narrow_phase::{ContactAlgorithm3};
use ncollide::shape::{Ball, Ball3, Cuboid, Plane, ShapeHandle3};
use ncollide::query::{self, Proximity};
use ncollide::events::{ContactEvents};


pub struct World {
    ecs: Ecs,
    player: Option<Entity>,
    pub camera: Option<Entity>,
    collision_world: CollisionWorld<Point, Isometry3<f32>, Entity>,
    shapes: HashMap<PhysicsShape, CollisionData>,
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
    groups.set_whitelist(&[1, 2]);
    groups.set_blacklist(&[]);
    map.insert(PhysicsShape::Chara, CollisionData {
        shape: ShapeHandle3::new(Ball::new(1.0)),
        groups: groups,
    });

    let mut groups = CollisionGroups::new();
    groups.set_membership(&[2]);
    groups.set_whitelist(&[1]);
    groups.set_blacklist(&[2]);
    map.insert(PhysicsShape::Wall, CollisionData {
        shape: ShapeHandle3::new(Cuboid::new(Vector3::new(0.5, 0.5, 0.5))),
        groups: groups,
    });

    map
}

impl World {
    pub fn new() -> Self {
        let mut collision_world = CollisionWorld::new(0.02);
        let mut world = World {
            ecs: Ecs::new(),
            player: None,
            camera: None,
            collision_world: collision_world,
            shapes: shape_handles(),
        };


        let player = world.spawn(prefab::mob("Dood"), Point::new(0.0, 0.0, 0.0));
        let camera = world.spawn(Loadout::new().c(Camera::new(player)), point::zero());

        world.player = Some(player);
        world.camera = Some(camera);
        world
    }

    // immut

    pub fn ecs(&self) -> &Ecs {
        &self.ecs
    }

    pub fn entities(&self) -> slice::Iter<Entity> {
        self.ecs.iter()
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
                self.ecs().positions.get(cam.following).cloned()
            }
        })
    }

    // mut

    pub fn ecs_mut(&mut self) -> &mut Ecs {
        &mut self.ecs
    }

    pub fn spawn(&mut self, mut loadout: Loadout, pos: Point) -> Entity {
        loadout = loadout.c(pos);

        let entity = loadout.make(&mut self.ecs);


        if self.ecs.physics.contains(entity) {
            let collision_data = {
                let phys = self.ecs.physics.get_or_err(entity);
                self.shapes.get(&phys.shape).cloned().unwrap()
            };
            let pos = self.ecs.positions.get_or_err(entity).clone();
            let mut ball_groups = CollisionGroups::new();
            ball_groups.set_membership(&[1]);
            let obj_pos = Isometry3::new(Vector3::new(pos.x, pos.y, pos.z), nalgebra::zero());
            let handle = self.collision_world.add(obj_pos,
                                                  collision_data.shape,
                                                  collision_data.groups,
                                                  GeometricQueryType::Contacts(0.0, 0.0),
                                                  entity);

            let mut phys = self.ecs.physics.get_mut_or_err(entity);
            phys.handle = Some(handle);
        }

        entity
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

    pub fn update_physics(&mut self) {
        self.update_world_to_physics();
        self.update_collision_world();
        self.update_physics_to_world();
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
                    let pos = Isometry3::new(Vector3::new(pos.x, pos.y, pos.z), nalgebra::zero());
                    self.collision_world.set_position(handle, pos);
                }
            }
        }

    }

    fn update_collision_world(&mut self) {
        self.collision_world.update();
    }

    fn update_physics_to_world(&mut self) {
        for (e1, e2, ca) in self.collision_world.contact_pairs() {
            let mut contacts = Vec::new();
            ca.contacts(&mut contacts);
            for contact in contacts {
                let mut move_vec = contact.normal.unwrap() * contact.depth * -0.5;
                {
                    if self.ecs.physics.has(*e1.data()) {
                        if let Some(p1) = self.ecs.positions.get_mut(*e1.data()) {
                            p1.x += move_vec.x;
                            p1.z += move_vec.z;
                        }
                    }
                }
                move_vec *= -1.0;
                {
                    if self.ecs.physics.has(*e2.data()) {
                        if let Some(p2) = self.ecs.positions.get_mut(*e2.data()) {
                            p2.x += move_vec.x;
                            p2.z += move_vec.z;
                        }
                    }
                }
            }
        }
    }
}
