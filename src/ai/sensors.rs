use std::collections::HashMap;

use alga::linear::EuclideanSpace;
use calx_ecs::Entity;

use debug;
use ecs::traits::*;
use world::World;

use super::{Ai, AiFacts, Target};

macro_rules! generate_sensors {
    ( $( $prop:ident, $default:expr, $sensor:ident );+ $(;)*) => {
        macro_attr! {
        #[derive(Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, PartialEq, Debug, Clone, EnumFromStr!)]
        pub enum AiProp {
            $(
                $prop,
            )*
        }
        }

        pub(super) fn default_ai_facts() -> AiFacts {
            let mut facts = AiFacts::new();
            $(
                facts.insert(AiProp::$prop, $default);
            )*;
            facts
        }


        pub(super) fn all_props() -> Vec<AiProp> {
            vec![
                $(
                    AiProp::$prop,
                )*
            ]
        }

        pub fn make_sensors() -> HashMap<AiProp, Sensor> {
            let mut results = HashMap::new();
            $(
                results.insert(AiProp::$prop, Sensor::new($sensor));
            )*;
            results
        }
    }
}

generate_sensors! {
    HasTarget, false, sense_has_target;
    // TargetVisible, false, sense_target_visible;
    TargetDead, false, sense_target_dead;

    HealthLow, false, sense_health_low;

    OnTopOfTarget, false, sense_on_top_of_target;
    // PathToTargetClear, false, sense_path_to_target_clear;

    TargetInRange, false, sense_target_in_range;
    TargetClose, false, sense_target_close;

    Exists, true, sense_always_true;
    Moving, false, sense_always_false;
}

// fn sense_has_throwable(world: &World, entity: Entity, ai: &Ai) -> bool {
//     entity.inventory(world)
//           .iter()
//           .any(|item| item.basename(world) == "watermelon")
// }
// 
// fn sense_target_visible(world: &World, entity: Entity, ai: &Ai) -> bool {
//     ai.data.targets.borrow().peek().map_or(false, |t| {
//         if t.entity.is_none() {
//             return false;
//         }
// 
//         let pos = match world.position(t.entity.unwrap()) {
//             Some(t) => t,
//             None => return false,
//         };
// 
//         entity.has_los(pos, world, Some(6))
//     })
// }

fn sense_target_dead(world: &World, _entity: Entity, ai: &Ai) -> bool {
    false
}

// fn sense_next_to_target(world: &World, entity: Entity, ai: &Ai) -> bool {
//     ai.data.targets.borrow().peek().map_or(false, |t| {
//         if t.entity.is_none() {
//             return false;
//         }
//         let pos = match world.position(t.entity.unwrap()) {
//             Some(p) => p,
//             None => return false,
//         };
// 
//         world.position(entity).unwrap().is_next_to(pos)
//     })
// }
// 
// 
// fn sense_path_to_target_clear(world: &World, entity: Entity, ai: &Ai) -> bool {
//     ai.data.targets.borrow().peek().map_or(false, |t| {
//         if t.entity.is_none() {
//             return false;
//         }
// 
//         let target_pos = match world.position(t.entity.unwrap()) {
//             Some(p) => p,
//             None => return false,
//         };
// 
//         let my_pos = world.position(entity).unwrap();
//         let is_item = world.is_item(t.entity.unwrap());
// 
//         let path = Path::find(my_pos, target_pos, world, Walkability::MonstersBlocking);
//         if is_item {
//             path.len() > 0 && world.mob_at(target_pos).is_none()
//         } else {
//             path.len() > 0
//         }
//     })
// }

fn target_within_dist(world: &World, entity: Entity, target: &Target, dist: f32) -> bool {
        let pos = match target.position(world) {
            Some(p) => p,
            None => return false,
        };

    let dist_to_target = pos.distance(&world.position(entity).unwrap().pos);
    log!("{}", dist_to_target);
    dist_to_target < dist
}

fn sense_on_top_of_target(world: &World, entity: Entity, ai: &Ai) -> bool {
    ai.data.targets.borrow().peek().map_or(false, |t| {
        target_within_dist(world, entity, t, debug::get("ai_on_top"))
    })
}

fn sense_target_in_range(world: &World, entity: Entity, ai: &Ai) -> bool {
    ai.data.targets.borrow().peek().map_or(false, |t| {
        target_within_dist(world, entity, t, debug::get("ai_in_range"))
    })
}

fn sense_target_close(world: &World, entity: Entity, ai: &Ai) -> bool {
    ai.data.targets.borrow().peek().map_or(false, |t| {
        target_within_dist(world, entity, t, debug::get("ai_close"))
    })
}

// fn sense_target_in_inventory(world: &World, entity: Entity, ai: &Ai) -> bool {
//     ai.data.targets.borrow().peek().map_or(false, |t| {
//         if t.entity.is_none() {
//             return false;
//         }
//         let e = world.entities_in(entity);
//         e.contains(&t.entity.unwrap())
//     })
// }

fn sense_has_target(_world: &World, _entity: Entity, ai: &Ai) -> bool {
    !ai.data.targets.borrow().is_empty()
}

fn sense_health_low(world: &World, entity: Entity, _ai: &Ai) -> bool {
    world.ecs()
         .healths
         .map_or(false, |h| h.percent() < debug::get("ai_health_low"), entity)
}

// fn sense_found_item(world: &World, entity: Entity, _ai: &Ai) -> bool {
//     world.seen_entities(entity)
//          .iter()
//          .any(|i| world.is_item(*i))
// }
// 
// fn sense_throwable_nearby(world: &World, entity: Entity, ai: &Ai) -> bool {
//     world.seen_entities(entity)
//          .iter()
//          .any(|i| world.is_item(*i) && i.basename(world) == "watermelon")
// }

fn sense_always_true(_world: &World, _entity: Entity, _ai: &Ai) -> bool {
    true
}

fn sense_always_false(_world: &World, _entity: Entity, _ai: &Ai) -> bool {
    false
}


pub struct Sensor {
    pub callback: Box<Fn(&World, Entity, &Ai) -> bool>,
}

impl Sensor {
    pub fn new<F>(callback: F) -> Self
    where
        F: 'static + Fn(&World, Entity, &Ai) -> bool,
    {
        Sensor { callback: Box::new(callback) }
    }
}

trait Sense {
    fn sense(world: &World, entity: Entity, ai: &Ai) -> bool;
}
