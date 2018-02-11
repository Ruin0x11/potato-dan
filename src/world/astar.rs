use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::f32;

use ncollide::world::{CollisionObjectHandle, CollisionGroups, CollisionObject3, CollisionWorld, GeometricQueryType};
use nalgebra::{self, Isometry3, Point3, Translation3, Vector3, Matrix3x1};
use ncollide::shape::{Cuboid, ShapeHandle3};
use ncollide::query::{self, Proximity};
use ncollide::events::{ContactEvents};
use ncollide::bounding_volume::AABB;

use ecs::traits::*;
use point::*;
use super::{World, CollideWorld, CollisionDataExtra};

const CALCULATION_LIMIT: u32 = 150;

#[derive(Clone, Copy, Debug)]
pub struct Node {
    pub sensor: CollisionObjectHandle,
    pub blocked: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct State {
    cost: f32,
    position: Point2d,
}

impl Eq for State {}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        assert!(self.cost.is_finite());
        assert!(other.cost.is_finite());
        if other.cost > self.cost {
            Ordering::Greater
        } else if other.cost < self.cost {
            Ordering::Less
        } else if other.cost == self.cost {
            Ordering::Equal
        } else {
            unreachable!()
        }
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &State) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Grid {
    groups: CollisionGroups,
    pub nodes: HashMap<Point2d, bool>,
    pub size: (u32, u32),
}

impl Grid {
    pub fn new(world: &mut CollideWorld, size: (u32, u32)) -> Self {
        let mut nodes = HashMap::new();
        let size = (size.0 * 2, size.1 * 2);

        for x in 0..size.0 {
            for z in 0..size.1 {
                let pos = Point2d::new(x as i32, z as i32);
                nodes.insert(pos, false);
            }
        }

        let mut groups = CollisionGroups::new();
        groups.set_membership(&[4]);
        groups.set_whitelist(&[1, 2, 3]);
        groups.set_blacklist(&[4]);

        Grid {
            groups: groups,
            nodes: nodes,
            size: size,
        }
    }

    pub fn discretize(&mut self, world: &CollideWorld) {
        for x in 0..self.size.0 {
            for z in 0..self.size.1 {
                let pos = Point2d::new(x as i32, z as i32);
                let mins = Point::new((x) as f32 + 0.2, -10.0, (z) as f32 + 0.2);
                let maxs = Point::new((x) as f32 + 0.8,  10.0, (z) as f32 + 0.8);
                let aabb = AABB::new(mins, maxs);
                let mut i = world.interferences_with_aabb(&aabb, &self.groups);
                let blocked = i.next().is_some();
                *self.nodes.get_mut(&pos).unwrap() = blocked;
            }
        }
    }

    fn neighbors(&self, center: Point2d) -> Vec<Point2d> {
        if !self.nodes.contains_key(&center) {
            return Vec::new();
        }

        let nearby_points: [Point2d; 9] = [
            Point2d::new(-1, -1),
            Point2d::new(-1,  0),
            Point2d::new(-1,  1),
            Point2d::new( 0, -1),
            Point2d::new( 0,  0),
            Point2d::new( 0,  1),
            Point2d::new( 1, -1),
            Point2d::new( 1,  0),
            Point2d::new( 1,  1),
        ];

        nearby_points.iter()
            .map(|&d| center + d.coords)
            .filter(|point| !self.blocked(point))
            .collect::<Vec<_>>()
    }

    fn blocked(&self, at: &Point2d) -> bool {
        self.nodes.get(at).map_or(true, |blocked| *blocked)
    }
}

fn search_heuristic(destination: Point2d, next: Point2d) -> f32 {
    ((destination.x - next.x).abs() + (destination.y - next.y).abs()) as f32
}

fn cost_heuristic(current: Point2d, next: Point2d) -> f32 {
    assert!((current.x - next.x).abs() <= 1);
    assert!((current.y - next.y).abs() <= 1);
    1.0
}

fn create_path(from: Point2d, to: Point2d, came_from: HashMap<Point2d, Option<Point2d>>) -> Vec<Point2d> {
    let mut current = to;
    let mut path_buffer = vec![current];
    while current != from {
        match came_from.get(&current) {
            Some(&Some(new_current)) => {
                current = new_current;
                if current != from {
                    path_buffer.push(current);
                }
            }
            Some(&None) => panic!(
                "Every point except for the initial one (`from`) one should be some."),
            None => {
                path_buffer = vec![];
                break
            },
        }
    }

    assert_eq!(None, path_buffer.iter().find(|&&p| p == from));

    path_buffer
}

pub fn find_path(from: Point2d, to: Point2d, grid: &Grid) -> Vec<Point2d> {
    if from == to {
        return vec![];
    }

    let mut frontier = BinaryHeap::new();
    frontier.push(State { position: from, cost: 0.0 });
    let mut came_from = HashMap::new();
    let mut cost_so_far = HashMap::new();

    came_from.insert(from, None);
    cost_so_far.insert(from, 0.0);

    // NOTE: the map is effectively infinite. We need to limit the
    // calculations or the algorithm will try to explore the
    // entire world before it decides that no path exists.
    let mut calculation_steps = 0;

    while let Some(current) = frontier.pop() {
        if current.position == to {
            break
        }

        if calculation_steps >= CALCULATION_LIMIT {
            break
        } else {
            calculation_steps += 1;
        }
        let neigh = grid.neighbors(current.position);

        for &next in neigh.iter() {
            let new_cost = cost_so_far[&current.position] + cost_heuristic(current.position, next);
            let val = cost_so_far.entry(next).or_insert(f32::MAX);
            if new_cost < *val {
                *val = new_cost;
                let priority = new_cost + search_heuristic(to, next);
                frontier.push(State { position: next, cost: priority });
                came_from.insert(next, Some(current.position));
            }
        }
    }

    create_path(from, to, came_from)
}
