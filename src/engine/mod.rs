pub mod keys;
mod glium;

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct MouseState {
    pub pos: (i32, i32),
    pub pressed: (bool, bool, bool),
    pub wheel: f32,
}
