use GameContext;
use engine::keys::{Key, KeyCode};
use world::World;
use ecs::prefab;
use ecs::traits::*;
use point::*;
use calx_ecs::Entity;

pub struct GameState {
    pub world: World,
    pub player: Entity,
}

impl GameState {
    pub fn new() -> Self {
        let mut world = World::new();
        let player = world.spawn(prefab::mob("Dood"), Point::new(0.0, 0.0, 0.0));
        GameState {
            world: world,
            player: player,
        }
    }
}

/// A bindable command that can be executed by the player.
pub enum Command {
    Move(Direction),
    Wait,
    Quit,
}

impl From<Key> for Command {
    fn from(key: Key) -> Command {
        match key {
            Key { code: KeyCode::Escape, .. } => Command::Quit,
            Key { code: KeyCode::Left, .. } |
            Key { code: KeyCode::H, .. } |
            Key { code: KeyCode::NumPad4, .. } => Command::Move(Direction::W),
            Key { code: KeyCode::Right, .. } |
            Key { code: KeyCode::L, .. } |
            Key { code: KeyCode::NumPad6, .. } => Command::Move(Direction::E),
            Key { code: KeyCode::Up, .. } |
            Key { code: KeyCode::K, .. } |
            Key { code: KeyCode::NumPad8, .. } => Command::Move(Direction::N),
            Key { code: KeyCode::Down, .. } |
            Key { code: KeyCode::J, .. } |
            Key { code: KeyCode::NumPad2, .. } => Command::Move(Direction::S),
            Key { code: KeyCode::B, .. } |
            Key { code: KeyCode::NumPad1, .. } => Command::Move(Direction::SW),
            Key { code: KeyCode::N, .. } |
            Key { code: KeyCode::NumPad3, .. } => Command::Move(Direction::SE),
            Key { code: KeyCode::Y, .. } |
            Key { code: KeyCode::NumPad7, .. } => Command::Move(Direction::NW),
            Key { code: KeyCode::U, .. } |
            Key { code: KeyCode::NumPad9, .. } => Command::Move(Direction::NE),

            _ => Command::Wait,
        }
    }
}

pub fn game_step(context: &mut GameContext, input: Option<Key>) {
    if let Some(key) = input {
        let command = Command::from(key);
        run_command(context, command);
    }
}

pub fn run_command(context: &mut GameContext, command: Command) {
    match command {
        Command::Move(dir) => {
            let player = context.state.player;

            let mut pos = context.state.world.ecs_mut().positions.get_mut_or_err(player);
            pos.direction = dir;
        },
        Command::Quit => println!("can't quit"),
        Command::Wait => (),

    }
}
