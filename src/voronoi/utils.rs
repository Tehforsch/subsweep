use super::Point;
use crate::prelude::Float;

pub fn sign(p1: Point, p2: Point, p3: Point) -> Float {
    (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
}