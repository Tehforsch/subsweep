use super::FaceIndex;
use super::Point;
use super::PointIndex;
use super::TetraIndex;
use crate::prelude::Float;

#[cfg(not(feature = "2d"))]
#[derive(Clone)]
pub struct Tetra {
    pub p1: PointIndex,
    pub p2: PointIndex,
    pub p3: PointIndex,
    pub p4: PointIndex,
    pub f1: TetraFace,
    pub f2: TetraFace,
    pub f3: TetraFace,
    pub f4: TetraFace,
}

#[cfg(not(feature = "2d"))]
#[derive(Clone)]
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

#[derive(Clone, Debug)]
pub struct Triangle {
    pub p1: PointIndex,
    pub p2: PointIndex,
    pub p3: PointIndex,
    pub f1: TetraFace,
    pub f2: TetraFace,
    pub f3: TetraFace,
}

impl Tetra {
    pub fn find_face(&self, face: FaceIndex) -> &TetraFace {
        self.iter_faces().find(|f| f.face == face).unwrap()
    }

    pub fn find_face_mut(&mut self, face: FaceIndex) -> &mut TetraFace {
        self.iter_faces_mut().find(|f| f.face == face).unwrap()
    }

    pub fn find_face_opposite(&self, p: PointIndex) -> &TetraFace {
        self.iter_points()
            .zip(self.iter_faces())
            .find(|(point, _)| **point == p)
            .map(|(_, face)| face)
            .unwrap_or_else(|| {
                panic!("find_face_opposite called with point that is not part of the tetra.");
            })
    }

    pub fn find_point_opposite(&self, f: FaceIndex) -> PointIndex {
        self.iter_faces()
            .zip(self.iter_points())
            .find(|(face, _)| face.face == f)
            .map(|(_, point)| *point)
            .unwrap_or_else(|| {
                panic!("find_point_opposite called with face that is not part of the tetra.");
            })
    }

    pub fn get_common_face_with(&self, other: &Tetra) -> Option<FaceIndex> {
        self.iter_faces()
            .flat_map(move |f_self| other.iter_faces().map(move |f_other| (f_self, f_other)))
            .find(|(fa, fb)| fa.face == fb.face)
            .map(|(fa, _)| fa.face)
    }
}

#[cfg(feature = "2d")]
impl Triangle {
    pub fn iter_faces(&self) -> impl Iterator<Item = &TetraFace> {
        ([&self.f1, &self.f2, &self.f3]).into_iter()
    }

    pub fn iter_points(&self) -> impl Iterator<Item = &PointIndex> {
        ([&self.p1, &self.p2, &self.p3]).into_iter()
    }

    pub fn iter_faces_mut(&mut self) -> impl Iterator<Item = &mut TetraFace> {
        ([&mut self.f1, &mut self.f2, &mut self.f3]).into_iter()
    }
}

#[cfg(feature = "3d")]
impl Tetra {
    pub fn iter_faces(&self) -> impl Iterator<Item = &TetraFace> {
        ([&self.f1, &self.f2, &self.f3, &self.f4]).into_iter()
    }

    pub fn iter_points(&self) -> impl Iterator<Item = &PointIndex> {
        ([&self.p1, &self.p2, &self.p3, &self.p4]).into_iter()
    }

    pub fn iter_faces_mut(&mut self) -> impl Iterator<Item = &mut TetraFace> {
        ([&mut self.f1, &mut self.f2, &mut self.f3, &mut self.f4]).into_iter()
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

pub fn sign(p1: Point, p2: Point, p3: Point) -> Float {
    (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
}

#[cfg(feature = "2d")]
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

#[cfg(feature = "3d")]
impl TetraData {
    pub fn contains(&self, _point: Point) -> bool {
        todo!()
    }

    pub fn circumcircle_contains(&self, _point: Point) -> bool {
        todo!()
    }

    pub fn _is_positively_oriented(&self) -> bool {
        todo!()
    }

    pub fn get_center_of_circumcircle(&self) -> Point {
        todo!()
    }
}

#[cfg(feature = "2d")]
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
