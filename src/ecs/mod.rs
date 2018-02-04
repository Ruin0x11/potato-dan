pub mod components;
pub mod prefab;
pub mod traits;
use point;

Ecs! {
    healths: components::Health,
    names: components::Name,
    physics: components::Physics,
    positions: point::Point,
    charas: components::Chara,
    cameras: components::Camera,
    appearances: components::Appearance,
}
