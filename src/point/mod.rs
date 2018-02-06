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


pub fn rotate_around(point: Point, pivot: Point, rot: f32) -> Point {
    let now_x = (pivot.x + (point.x - pivot.x) * (-rot).cos() + (point.z - pivot.z) * (-rot).sin());
    let now_z = (pivot.z + (point.z - pivot.z) * (-rot).cos() - (point.x - pivot.x) * (-rot).sin());
    Point::new(now_x, pivot.y, now_z)
}

pub fn relative(point: Point, offset: Point, rot: f32) -> Point {
    let front = point + offset.coords;
    rotate_around(front, point, rot)
}
