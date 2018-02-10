use calx_ecs::Entity;

use ai::*;
use ecs::traits::ComponentQuery;
use world::World;

#[derive(Eq, PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum AiKind {
    Wait,
    SeekTarget,
    Follow,
    Guard,
}

impl AiKind {
    pub fn on_goal(&self, goal: AiGoal, entity: Entity, world: &mut World) {
        match *self {
            AiKind::Guard => {
                match goal {
                    //AiGoal::KillTarget => {
                    //    format_mes!(world, entity, "%u: Scum!");
                    //},
                    _ => (),
                }
            },
            _ => (),
        }
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum AiGoal {
    Wander,
    KillTarget,

    DoNothing,
}

impl AiGoal {
    pub fn get_end_state(&self) -> AiFacts {
        let mut goal = AiFacts::new();
        for (prop, val) in self.get_props() {
            goal.insert(prop, val);
        }
        goal
    }

    fn get_props(&self) -> Vec<(AiProp, bool)> {
        // TODO: instead make the "health low" things triggers for entering the new goal of "run
        // away and heal"
        match *self {
            AiGoal::Wander => vec![(AiProp::Moving, true)],
            AiGoal::DoNothing => vec![(AiProp::Exists, false)],
            AiGoal::KillTarget => vec![(AiProp::TargetDead, true), (AiProp::HealthLow, false)],
        }
    }

    pub fn requires_target(&self) -> bool {
        match *self {
            AiGoal::KillTarget => true,
            _ => false,
        }
    }

    pub fn requires_position(&self) -> bool {
        match *self {
            _ => false,
        }
    }
}

fn get_default_goal(entity: Entity, world: &World) -> Target {
    let ai_compo = world.ecs().ais.get_or_err(entity);
    let ai = &ai_compo.data;

    match world.player() {
        Some(p) => attack_target(p),
        None => Target::new(AiGoal::Wander),
    }
}

fn attack_target(entity: Entity) -> Target {
    Target {
        obj: TargetObject::Entity(entity),
        priority: 100,
        goal: AiGoal::KillTarget,
    }
}

pub fn make_new_plan(entity: Entity, world: &World) -> (AiFacts, Option<Target>) {
    let ai = &world.ecs().ais.get_or_err(entity).data;

    let mut made_target = false;
    let target = match ai.targets.borrow().peek() {
        Some(next_target) => *next_target,
        None => {
            made_target = true;
            get_default_goal(entity, world)
        },
    };

    log!("New AI target: {:?}", target);
    let desired = target.goal.get_end_state();
    if made_target {
        (desired, Some(target))
    } else {
        (desired, None)
    }
}
