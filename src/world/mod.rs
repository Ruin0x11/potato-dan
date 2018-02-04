use std::slice;

use calx_ecs::Entity;
use ecs::*;
use ecs::prefab;
use ecs::Loadout;
use ecs::traits::*;
use ecs::components::*;
use point::*;

pub struct World {
    ecs: Ecs,
    player: Option<Entity>,
    pub camera: Option<Entity>,
}

impl World {
    pub fn new() -> Self {
        let mut world = World {
            ecs: Ecs::new(),
            player: None,
            camera: None,
        };

        let player = world.spawn(prefab::mob("Dood"), Point::new(5.0, 5.0, 0.0));
        let camera = world.spawn(Loadout::new().c(Camera::new(player))), POINT_ZERO);

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

    pub fn contains(&self, entity: Entity) -> bool {
        self.ecs.contains(entity)
    }

    pub fn camera_pos(&self) -> Option<Point> {
        self.camera.map(|c| self.ecs().cameras.get_or_err(c)).and_then(|cam| {
            println!("ok");
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

        entity
    }

    pub fn kill(&mut self, entity: Entity) {
        self.ecs_mut().healths.map_mut(|h| h.kill(), entity);
    }
}
