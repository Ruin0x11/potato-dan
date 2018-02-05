use calx_ecs::Entity;
use rand::{self, Rng};

use ecs::traits::*;
use point::*;
use world::World;

use ncollide::world::CollisionObjectHandle;

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

    pub fn hurt(&mut self, amount: i32) {
        self.hit_points -= amount;
    }

    pub fn kill(&mut self) {
        self.hit_points = 0;
    }

    pub fn is_dead(&self) -> bool {
        self.hit_points <= 0
    }
}

fn none() -> Option<CollisionObjectHandle> {
    None
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PhysicsShape {
    Chara,
    Wall,
    Bullet,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PhysicsKind {
    Physical,
    Bullet,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub pos: Point,
    pub dir: Direction,
}

impl Position {
    pub fn new(pos: Point) -> Self {
        Position {
            pos: pos,
            dir: Direction::S,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Physics {
    pub dx: f32,
    pub dy: f32,
    pub dz: f32,
    pub accel_x: f32,
    pub accel_y: f32,
    pub accel_z: f32,
    pub movement_frames: u32,
    pub shape: PhysicsShape,
    pub kind: PhysicsKind,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    #[serde(default = "none")]
    pub handle: Option<CollisionObjectHandle>,
}

impl Physics {
    pub fn new(shape: PhysicsShape, kind: PhysicsKind) -> Self {
        Physics {
            dx: 0.0,
            dy: 0.0,
            dz: 0.0,
            accel_x: 0.0,
            accel_y: 0.0,
            accel_z: 0.0,
            movement_frames: 0,
            shape: shape,
            kind: kind,
            handle: None,
        }
    }

    pub fn impulse(&mut self, direction: Point) {
        self.dx += direction.x;
        self.dy += direction.y;
        self.dz += direction.z;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Camera {
    pub primary: bool,
    pub following: Entity,
}

impl Camera {
    pub fn new(following: Entity) -> Self {
        Camera {
            primary: true,
            following: following,
        }
    }

    pub fn pos(self, world: &World) -> Option<Point> {
        if !world.contains(self.following) {
            return None;
        }
        Some(world.ecs().positions.get_or_err(self.following).pos.clone())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CharaAppearance {
    pub body_kind: u32,
    pub feet_kind: u32,
    pub jacket_kind: u32,
    pub hair_kind: u32,
    pub helmet_kind: u32,
    pub ear_kind: u32,
    pub tail_kind: u32,

    pub face_kind: u32
}

impl CharaAppearance {
    pub fn new() -> Self {
        CharaAppearance {
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
    pub fn new_random() -> Self {
        let mut rng = rand::thread_rng();
        CharaAppearance {
            body_kind: rng.next_u32(),
            feet_kind: 0,
            jacket_kind: rng.next_u32(),
            hair_kind: rng.next_u32(),
            helmet_kind: rng.next_u32(),
            ear_kind: rng.next_u32(),
            tail_kind: rng.next_u32(),

            face_kind: rng.next_u32(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectAppearance {
    pub kind: String,
    pub offset: (i32, i32),
    pub variant: u32,
    pub directional: bool,
}

impl ObjectAppearance {
    pub fn new(kind: &str, offset: (i32, i32), variant: u32) -> Self {
        ObjectAppearance {
            kind: kind.to_string(),
            offset: offset,
            variant: variant,
            directional: false
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Appearance {
    Chara(CharaAppearance),
    Object(ObjectAppearance),
    Bullet,
}

impl Appearance {
    pub fn new_chara() -> Self {
        Appearance::Chara(CharaAppearance::new_random())
    }

    pub fn new(kind: &str, offset: (i32, i32), variant: u32) -> Self {
        Appearance::Object(ObjectAppearance::new(kind, offset, variant))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bullet {
    pub damage: i32,
    pub time_left: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum BulletKind {
    NineMm,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gun {
    pub chara: Option<Entity>,
    pub bullet: BulletKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chara {
    pub gun: Option<Entity>
}

impl Chara {
    pub fn new() -> Self {
        Chara {
            gun: None
        }
    }
}
