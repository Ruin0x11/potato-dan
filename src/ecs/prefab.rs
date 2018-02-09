use calx_ecs::Entity;
use debug;
use ecs::Loadout;
use ecs::components::*;

pub fn mob(name: &str) -> Loadout {
    Loadout::new()
        .c(Appearance::new_chara())
        .c(Name::new(name))
        .c(Health::new(debug::get("health") as i32))
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
    let gun = Gun::new(
        BulletKind::NineMm,
        debug::get("spread"),
        debug::get("clip_size") as u16,
        debug::get("fire_rate") as u16,
        debug::get("reload_time") as u16,
    );
    Loadout::new()
        .c(app)
        .c(gun)
}

pub fn bomb() -> Loadout {
    Loadout::new()
        .c(Appearance::new("wall", (0, -70), 0))
        .c(Bomb::new())
}

pub fn bullet(fired_by: Entity) -> Loadout {
    Loadout::new()
        .c(Appearance::Bullet)
        .c(Physics::new(PhysicsShape::Bullet, PhysicsKind::Bullet))
        .c(Bullet { damage: debug::get("bullet_damage") as i32, time_left: debug::get("bullet_time"), fired_by: fired_by })
}

pub fn wall() -> Loadout {
    Loadout::new()
        .c(Appearance::new("wall", (0, -70), 0))
        .c(Health::new(100))
        .c(Physics::new(PhysicsShape::Wall, PhysicsKind::Physical))
}
