pub mod components;
pub mod prefab;
pub mod traits;

Ecs! {
    healths: components::Health,
    names: components::Name,
    positions: components::Position,
    appearances: components::Appearance,
}
