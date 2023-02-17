use super::FaceIndex;
use super::Point;
use super::PointIndex;
use super::TetraIndex;
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
    pub f1: TetraFace,
    pub f2: TetraFace,
    pub f3: TetraFace,
}

impl Triangle {
    pub fn find_face(&self, face: FaceIndex) -> &TetraFace {
        self.iter_faces().find(|f| f.face == face).unwrap()
    }

    pub fn find_face_mut(&mut self, face: FaceIndex) -> &mut TetraFace {
        self.iter_faces_mut().find(|f| f.face == face).unwrap()
    }

    pub fn find_face_opposite(&self, p: PointIndex) -> &TetraFace {
        if p == self.p1 {
            &self.f1
        } else if p == self.p2 {
            &self.f2
        } else if p == self.p3 {
            &self.f3
        } else {
            panic!("find_face_opposite called with point that is not part of the tetra.");
        }
    }

    pub fn find_point_opposite(&self, f: FaceIndex) -> PointIndex {
        if f == self.f1.face {
            self.p1
        } else if f == self.f2.face {
            self.p2
        } else if f == self.f3.face {
            self.p3
        } else {
            panic!("find_point_opposite called with face that is not part of the tetra.");
        }
    }

    pub fn iter_faces(&self) -> impl Iterator<Item = &TetraFace> {
        ([&self.f1, &self.f2, &self.f3]).into_iter()
    }

    pub fn iter_faces_mut(&mut self) -> impl Iterator<Item = &mut TetraFace> {
        ([&mut self.f1, &mut self.f2, &mut self.f3]).into_iter()
    }

    pub fn get_common_face_with(&self, other: &Triangle) -> FaceIndex {
        [
            (self.f1, other.f1),
            (self.f1, other.f2),
            (self.f1, other.f3),
            (self.f2, other.f1),
            (self.f2, other.f2),
            (self.f2, other.f3),
            (self.f3, other.f1),
            (self.f3, other.f2),
            (self.f3, other.f3),
        ]
        .iter()
        .find(|(fa, fb)| fa.face == fb.face)
        .map(|(fa, _)| fa)
        .unwrap()
        .face
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TetraFace {
    pub face: FaceIndex,
    pub opposing: Option<ConnectionData>,
}

#[derive(Debug, Clone, Copy)]
pub struct ConnectionData {
    pub tetra: TetraIndex,
    pub point: PointIndex,
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
        // assert!(self.is_positively_oriented());
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

    pub fn get_center_of_circumcircle(&self) -> Point {
        let a = self.p1;
        let b = self.p2;
        let c = self.p3;
        let d = 2.0 * (a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y));
        Point {
            x: 1.0 / d
                   * ((a.x.powi(2) + a.y.powi(2)) * (b.y - c.y)
                    + (b.x.powi(2) + b.y.powi(2)) * (c.y - a.y)
                    + (c.x.powi(2) + c.y.powi(2)) * (a.y - b.y)),
            y: 1.0 / d
                   * ((a.x.powi(2) + a.y.powi(2)) * (c.x - b.x)
                    + (b.x.powi(2) + b.y.powi(2)) * (a.x - c.x)
                    + (c.x.powi(2) + c.y.powi(2)) * (b.x - a.x)),
        }
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
