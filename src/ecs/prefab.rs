use calx_ecs::Entity;
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
        .c(Gun { bullet: BulletKind::NineMm, spread: 0.4, fire_rate: 1.0, clip_size: 100,
    reload_time: 5.0 })
}

pub fn bullet(fired_by: Entity) -> Loadout {
    Loadout::new()
        .c(Appearance::Bullet)
        .c(Physics::new(PhysicsShape::Bullet, PhysicsKind::Bullet))
        .c(Bullet { damage: 10, time_left: 60.0, fired_by: fired_by })
}

pub fn wall() -> Loadout {
    Loadout::new()
        .c(Appearance::new("wall", (0, -70), 0))
        .c(Health::new(100))
        .c(Physics::new(PhysicsShape::Wall, PhysicsKind::Physical))
}
