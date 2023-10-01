use std::iter::Zip;

use super::face_info::FaceInfo;
use super::FaceIndex;
use super::Point;
use super::PointIndex;
use crate::dimension::Dimension;
use crate::extent::Extent;
use crate::prelude::Float;
use crate::voronoi::visualizer::Visualizable;

pub trait DDimension: Dimension {
    type Face: Clone + DFace<Dimension = Self>;
    type FaceData: Clone + DFaceData<Dimension = Self>;
    type Tetra: Clone + DTetra<Dimension = Self>;
    type TetraData: DTetraData<Dimension = Self> + Clone + Visualizable;
    type VoronoiFaceData: std::fmt::Debug;
}

pub trait DTetra: core::fmt::Debug {
    type Dimension: DDimension;

    type PointsIter: Iterator<Item = PointIndex>;
    type FacesIter<'a>: Iterator<Item = &'a FaceInfo>
    where
        Self: 'a;
    type FacesMutIter<'a>: Iterator<Item = &'a mut FaceInfo>
    where
        Self: 'a;

    fn points(&self) -> Self::PointsIter;
    fn faces(&self) -> Self::FacesIter<'_>;
    fn faces_mut(&mut self) -> Self::FacesMutIter<'_>;

    fn contains_point(&self, p1: PointIndex) -> bool;

    fn faces_and_points(&self) -> Zip<Self::FacesIter<'_>, Self::PointsIter> {
        self.faces().zip(self.points())
    }

    fn find_face(&self, face: FaceIndex) -> &FaceInfo {
        self.faces().find(|f| f.face == face).unwrap()
    }

    fn find_face_mut(&mut self, face: FaceIndex) -> &mut FaceInfo {
        self.faces_mut().find(|f| f.face == face).unwrap()
    }

    fn find_face_opposite(&self, p: PointIndex) -> &FaceInfo {
        self.faces()
            .zip(self.points())
            .find(|(_, point)| *point == p)
            .map(|(face, _)| face)
            .unwrap_or_else(|| {
                panic!("find_face_opposite called with point that is not part of the tetra.");
            })
    }

    fn find_point_opposite(&self, f: FaceIndex) -> PointIndex {
        self.faces()
            .zip(self.points())
            .find(|(face, _)| face.face == f)
            .map(|(_, point)| point)
            .unwrap_or_else(|| {
                panic!("find_point_opposite called with face that is not part of the tetra.");
            })
    }

    fn get_common_face_with(&self, other: &Self) -> Option<FaceIndex> {
        self.faces()
            .find(|f| other.faces().any(|f2| f.face == f2.face))
            .map(|face| face.face)
    }
}

pub trait DTetraData:
    core::fmt::Debug + FromIterator<<Self::Dimension as Dimension>::Point>
{
    type Dimension: DDimension;

    fn all_encompassing(extent: &Extent<Point<Self::Dimension>>) -> Self;
    fn extent(&self) -> Extent<Point<Self::Dimension>>;
    fn contains(&self, p: Point<Self::Dimension>, extent: &Extent<Point<Self::Dimension>>) -> bool;
    fn distance_to_point(&self, p: Point<Self::Dimension>) -> Float;
    fn circumcircle_contains(&self, point: <Self::Dimension as Dimension>::Point) -> bool;
    fn is_positively_oriented(&self) -> bool;
    fn get_center_of_circumcircle(&self) -> Point<Self::Dimension>;
}

pub trait DFace {
    type Dimension: DDimension;

    type PointsIter: Iterator<Item = PointIndex>;
    fn points(&self) -> Self::PointsIter;

    fn contains_point(&self, point: PointIndex) -> bool {
        self.points().any(|p| p == point)
    }
}

pub trait DFaceData: FromIterator<Point<Self::Dimension>> {
    type Dimension: DDimension;
}
