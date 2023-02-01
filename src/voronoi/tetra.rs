use generational_arena::Index;

use super::{PointIndex, Point};

#[cfg(not(feature = "2d"))]
pub struct Tetra {
    pub p1: Index,
    pub p2: Index,
    pub p3: Index,
    pub p4: Index,
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

pub struct Triangle {
    pub p1: Index,
    pub p2: Index,
    pub p3: Index,
}

pub struct TriangleData {
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
}

