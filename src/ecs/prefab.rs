use ecs::Loadout;
use ecs::components::*;

pub fn mob(name: &str) -> Loadout {
    Loadout::new()
        .c(Appearance::new_chara())
        .c(Name::new(name))
        .c(Health::new(100))
        .c(Physics::new(PhysicsShape::Chara, PhysicsKind::Physical))
        .c(Chara::new())
}

pub fn gun() -> Loadout {
    let mut app = Appearance::Object(ObjectAppearance {
        kind: "gun".to_string(),
        offset: (0, 0),
        variant: 5,
        directional: true
    }
    );
    Loadout::new()
        .c(app)
        .c(Gun { bullet: BulletKind::NineMm, chara: None })
}

pub fn bullet() -> Loadout {
    Loadout::new()
        .c(Appearance::Bullet)
        .c(Physics::new(PhysicsShape::Bullet, PhysicsKind::Bullet))
        .c(Bullet { damage: 1, time_left: 60.0 })
}

pub fn wall() -> Loadout {
    Loadout::new()
        .c(Appearance::new("wall", (0, -70), 0))
        .c(Health::new(100))
        .c(Physics::new(PhysicsShape::Wall, PhysicsKind::Physical))
}
