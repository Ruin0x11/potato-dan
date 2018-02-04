use point::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Name {
    pub name: String,
}

impl Name {
    pub fn new(name: &str) -> Self {
        Name { name: name.to_string() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Health {
    pub hit_points: i32,
    pub max_hit_points: i32,
}

impl Health {
    pub fn new(max: i32) -> Self {
        assert!(max > 0);
        Health {
            hit_points: max,
            max_hit_points: max,
        }
    }

    pub fn percent(&self) -> f32 {
        self.hit_points as f32 / self.max_hit_points as f32
    }

    pub fn hurt(&mut self, amount: u32) {
        self.hit_points -= amount as i32;
    }

    pub fn kill(&mut self) {
        self.hit_points = 0;
    }

    pub fn is_dead(&self) -> bool {
        self.hit_points <= 0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub pos: Point,
    pub direction: Direction,
    pub dx: f32,
    pub dy: f32,
    pub accel_x: f32,
    pub accel_y: f32,
    pub movement_frames: u32,
}

impl Position {
    pub fn new(pos: Point) -> Self {
        Position {
            pos: pos,
            direction: Direction::S,
            dx: 0.0,
            dy: 0.0,
            accel_x: 0.0,
            accel_y: 0.0,
            movement_frames: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chara {
    pub body_kind: u32,
    pub feet_kind: u32,
    pub jacket_kind: u32,
    pub hair_kind: u32,
    pub helmet_kind: u32,
    pub ear_kind: u32,
    pub tail_kind: u32,

    pub face_kind: u32
}

impl Chara {
    pub fn new() -> Self {
        Chara {
            body_kind: 6,
            feet_kind: 0,
            jacket_kind: 2,
            hair_kind: 16,
            helmet_kind: 1,
            ear_kind: 1,
            tail_kind: 1,

            face_kind: 5,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Appearance {
    kind: String,
    variant: u32,
}

impl Appearance {
    pub fn new(kind: &str, variant: u32) -> Self {
        Appearance {
            kind: kind.to_string(),
            variant: variant
        }
    }
}
