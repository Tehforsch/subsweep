use super::math::determinant4x4;
use super::tetra::TetraFace;
use super::Point;
use super::PointIndex;
use crate::prelude::Float;

#[derive(Clone)]
pub struct Tetra3d {
    pub p1: PointIndex,
    pub p2: PointIndex,
    pub p3: PointIndex,
    pub p4: PointIndex,
    pub f1: TetraFace,
    pub f2: TetraFace,
    pub f3: TetraFace,
    pub f4: TetraFace,
}

#[derive(Clone)]
pub struct Tetra3dData {
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
    pub p4: Point,
}

impl Tetra3d {
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

impl Tetra3dData {
    pub fn all_encompassing(_points: &[Point]) -> Tetra3dData {
        todo!()
    }

    pub fn contains(&self, _point: Point) -> bool {
        todo!()
    }

    pub fn circumcircle_contains(&self, _point: Point) -> bool {
        todo!()
    }

    pub fn is_positively_oriented(&self) -> bool {
        #[rustfmt::skip]
        let det = determinant4x4(
            1.0, self.p1.x, self.p1.y, self.p1.z,
            1.0, self.p2.x, self.p2.y, self.p2.z,
            1.0, self.p3.x, self.p3.y, self.p3.z,
            1.0, self.p4.x, self.p4.y, self.p4.z,
        );
        det > 0.0
    }

    pub fn get_center_of_circumcircle(&self) -> Point {
        todo!()
    }
}
