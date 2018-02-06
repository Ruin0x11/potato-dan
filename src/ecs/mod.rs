pub mod components;
pub mod prefab;
pub mod traits;
use ai;
use point;

Ecs! {
    positions: components::Position,
    healths: components::Health,
    names: components::Name,
    physics: components::Physics,
    ais: ai::Ai,
    charas: components::Chara,
    cameras: components::Camera,
    appearances: components::Appearance,
    bullets: components::Bullet,
    guns: components::Gun,
    holds: components::Holds,
}
