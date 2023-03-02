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
pub type Tetra = super::tetra_2d::Triangle;
#[cfg(feature = "2d")]
pub type TetraData = super::tetra_2d::TriangleData;

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
