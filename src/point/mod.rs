pub mod direction;
pub use self::direction::*;

use nalgebra::{Point3, Point2};

pub type Point = Point3<f32>;

pub fn zero() -> Point {
    Point::new(0.0, 0.0, 0.0)
}


pub type Point2d = Point2<i32>;
