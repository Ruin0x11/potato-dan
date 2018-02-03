use GameContext;
use engine::keys::Key;
use world::World;

pub struct GameState {
    pub world: World,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            world: World::new(),
        }
    }
}

pub fn game_step(context: &mut GameContext, key: Option<Key>) {
    println!("Key: {:?}", key);
}
