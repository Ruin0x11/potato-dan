use ecs::Loadout;
use ecs::components::*;

pub fn mob(name: &str) -> Loadout {
    Loadout::new()
        .c(Name::new(name))
        .c(Health::new(1000))
        .c(Appearance::new())
}
