pub mod direction;
pub use self::direction::*;

use std::cmp::{Ordering};
use std::fmt::{Display, Formatter, Error};
use std::ops::{Add, AddAssign, Div, Sub};

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub const POINT_ZERO: Point = Point { x: 0.0, y: 0.0, z: 0.0 };

impl Point {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Point { x: x, y: y, z: z }
    }
}

impl Into<Point> for (f32, f32, f32) {
    fn into(self) -> Point {
        Point {
            x: self.0,
            y: self.1,
            z: self.2,
        }
    }
}

impl Into<(f32, f32, f32)> for Point {
    fn into(self) -> (f32, f32, f32) {
        (self.x, self.y, self.z)
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self + Point::new(-rhs.x, -rhs.y, -rhs.z)
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, _other: &Point) -> Option<Ordering> {
        // NOTE: I don't know that's the difference between this one
        // and the more explicit fn below. So let's just crash here
        // and see if and when we ever hit this.
        unimplemented!();
    }

    fn lt(&self, other: &Point) -> bool {
        self.x < other.x && self.y < other.y
    }

    fn le(&self, other: &Point) -> bool {
        self.x <= other.x && self.y <= other.y
    }

    fn gt(&self, other: &Point) -> bool {
        self.x > other.x && self.y > other.y
    }

    fn ge(&self, other: &Point) -> bool {
        self.x >= other.x && self.y >= other.y
    }
}

impl Add<(f32, f32, f32)> for Point {
    type Output = Self;

    fn add(self, rhs: (f32, f32, f32)) -> Self {
        let rhs: Point = rhs.into();
        self + rhs
    }
}

impl AddAssign<(f32, f32, f32)> for Point {
    fn add_assign(&mut self, rhs: (f32, f32, f32)) {
        let rhs: Point = rhs.into();
        *self = self.add(rhs);
    }
}

impl Sub<(f32, f32, f32)> for Point {
    type Output = Self;

    fn sub(self, rhs: (f32, f32, f32)) -> Self {
        let rhs: Point = rhs.into();
        self - rhs
    }
}

impl PartialEq<(f32, f32, f32)> for Point {
    fn eq(&self, other: &(f32, f32, f32)) -> bool {
        let other: Point = (*other).into();
        self == &other
    }
}

impl PartialOrd<(f32, f32, f32)> for Point {
    fn partial_cmp(&self, other: &(f32, f32, f32)) -> Option<Ordering> {
        let other: Point = (*other).into();
        self.partial_cmp(&other)
    }

    fn lt(&self, other: &(f32, f32, f32)) -> bool {
        let other: Point = (*other).into();
        self < &other
    }

    fn le(&self, other: &(f32, f32, f32)) -> bool {
        let other: Point = (*other).into();
        self <= &other
    }

    fn gt(&self, other: &(f32, f32, f32)) -> bool {
        let other: Point = (*other).into();
        self > &other
    }

    fn ge(&self, other: &(f32, f32, f32)) -> bool {
        let other: Point = (*other).into();
        self >= &other
    }
}

impl Div<f32> for Point {
    type Output = Self;

    fn div(self, rhs: f32) -> Self {
        Point::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

