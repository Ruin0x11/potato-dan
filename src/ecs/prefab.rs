use ecs::Loadout;
use ecs::components::*;

pub fn mob(name: &str) -> Loadout {
    Loadout::new()
        .c(Appearance::new_chara())
        .c(Name::new(name))
        .c(Health::new(1000))
        .c(Physics::new())
        .c(Chara)
}

pub fn wall() -> Loadout {
    Loadout::new()
        .c(Appearance::new("wall", 0))
        .c(Health::new(10000))
        .c(Physics::new())
}
