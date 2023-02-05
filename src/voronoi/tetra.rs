use super::FaceIndex;
use super::Point;
use super::PointIndex;
use crate::prelude::Float;

#[cfg(not(feature = "2d"))]
pub struct Tetra {
    pub p1: PointIndex,
    pub p2: PointIndex,
    pub p3: PointIndex,
    pub p4: PointIndex,
}

#[cfg(not(feature = "2d"))]
pub struct TetraData {
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
    pub p4: Point,
}

#[cfg(feature = "2d")]
pub type Tetra = Triangle;
#[cfg(feature = "2d")]
pub type TetraData = TriangleData;

#[derive(Debug)]
pub struct Triangle {
    pub p1: PointIndex,
    pub p2: PointIndex,
    pub p3: PointIndex,
    pub f1: FaceIndex,
    pub f2: FaceIndex,
    pub f3: FaceIndex,
}

#[derive(Debug)]
pub struct TriangleData {
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
}

fn sign(p1: Point, p2: Point, p3: Point) -> Float {
    (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
}

impl TriangleData {
    pub fn contains(&self, point: Point) -> bool {
        let d1 = sign(point, self.p1, self.p2);
        let d2 = sign(point, self.p2, self.p3);
        let d3 = sign(point, self.p3, self.p1);

        let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
        let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);

        !(has_neg && has_pos)
    }
}
