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

    pub fn circumcircle_contains(&self, point: Point) -> bool {
        // See for example Springel (2009), doi:10.1111/j.1365-2966.2009.15715.x
        assert!(self.is_positively_oriented());
        let a = self.p1;
        let b = self.p2;
        let c = self.p3;
        let d = point;
        #[rustfmt::skip]
        let det = determinant(
            b.x - a.x, b.y - a.y, (b.x - a.x).powi(2) + (b.y - a.y).powi(2),
            c.x - a.x, c.y - a.y, (c.x - a.x).powi(2) + (c.y - a.y).powi(2),
            d.x - a.x, d.y - a.y, (d.x - a.x).powi(2) + (d.y - a.y).powi(2)
        );
        det < 0.0
    }

    pub fn is_positively_oriented(&self) -> bool {
        #[rustfmt::skip]
        let det = determinant(
            1.0, self.p1.x, self.p1.y,
            1.0, self.p2.x, self.p2.y,
            1.0, self.p3.x, self.p3.y,
        );
        det > 0.0
    }
}

fn determinant(
    a11: Float,
    a12: Float,
    a13: Float,
    a21: Float,
    a22: Float,
    a23: Float,
    a31: Float,
    a32: Float,
    a33: Float,
) -> Float {
    a11 * a22 * a33 + a12 * a23 * a31 + a13 * a21 * a32
        - a13 * a22 * a31
        - a12 * a21 * a33
        - a11 * a23 * a32
}
