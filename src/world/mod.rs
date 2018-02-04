use std::slice;

use calx_ecs::Entity;
use ecs::*;
use ecs::traits::*;
use point::Point;

pub struct World {
    ecs: Ecs,
}

impl World {
    pub fn new() -> Self {
        World {
            ecs: Ecs::new(),
        }
    }

    // immut

    pub fn ecs(&self) -> &Ecs {
        &self.ecs
    }

    pub fn entities(&self) -> slice::Iter<Entity> {
        self.ecs.iter()
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
