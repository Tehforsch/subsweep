use super::face_info::FaceInfo;
use super::FaceIndex;
use super::PointIndex;
use crate::voronoi::precision_error::PrecisionError;

pub trait Dimension {
    type Point: Clone + Copy;
    type Face: Clone + DimensionFace<Dimension = Self>;
    type FaceData: Clone + DimensionFaceData<Dimension = Self>;
    type Tetra: Clone + DimensionTetra<Dimension = Self>;
    type TetraData: DimensionTetraData<Dimension = Self> + Clone;
}

pub trait DimensionTetra {
    type Dimension: Dimension;

    fn points(&self) -> Box<dyn Iterator<Item = PointIndex> + '_>;
    fn faces(&self) -> Box<dyn Iterator<Item = &FaceInfo> + '_>;
    fn faces_mut(&mut self) -> Box<dyn Iterator<Item = &mut FaceInfo> + '_>;

    fn faces_and_points(&self) -> Box<dyn Iterator<Item = (&FaceInfo, PointIndex)> + '_> {
        Box::new(self.faces().zip(self.points()))
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

    fn contains_point(&self, p1: PointIndex) -> bool {
        self.points().any(|p| p == p1)
    }
}

pub trait DimensionTetraData: FromIterator<<Self::Dimension as Dimension>::Point> {
    type Dimension: Dimension;

    fn all_encompassing(points: &[<Self::Dimension as Dimension>::Point]) -> Self;
    fn contains(&self, p: <Self::Dimension as Dimension>::Point) -> Result<bool, PrecisionError>;
    fn circumcircle_contains(
        &self,
        point: <Self::Dimension as Dimension>::Point,
    ) -> Result<bool, PrecisionError>;
    fn is_positively_oriented(&self) -> Result<bool, PrecisionError>;
    fn get_center_of_circumcircle(&self) -> <Self::Dimension as Dimension>::Point;
}

pub trait DimensionFace {
    type Dimension: Dimension;

    fn points(&self) -> Box<dyn Iterator<Item = PointIndex>>;

    fn contains_point(&self, point: PointIndex) -> bool {
        self.points().any(|p| p == point)
    }
}

pub trait DimensionFaceData: FromIterator<<Self::Dimension as Dimension>::Point> {
    type Dimension: Dimension;
}