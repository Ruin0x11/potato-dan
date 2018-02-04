pub mod direction;
pub use self::direction::*;

use nalgebra::Point3;

pub type Point = Point3<f32>;

pub fn zero() -> Point {
    Point::new(0.0, 0.0, 0.0)
}
