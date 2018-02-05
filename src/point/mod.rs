pub mod direction;
pub use self::direction::*;

use nalgebra::{Point3, Point2};

pub type Point = Point3<f32>;

pub fn zero() -> Point {
    Point::new(0.0, 0.0, 0.0)
}

pub type Point2d = Point2<i32>;

pub fn angle<I: Into<(i32, i32)>>(a: I, b: I) -> f32 {
    let a = a.into();
    let b = b.into();
    let y = (b.1 - a.1) as f32;
    let x = (b.0 - a.0) as f32;
    y.atan2(x)
}
