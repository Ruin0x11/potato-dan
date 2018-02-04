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
pub struct Appearance {
}

impl Appearance {
    pub fn new() -> Self {
        Appearance {
            
        }
    }
}
