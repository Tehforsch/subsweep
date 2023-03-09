use super::FaceIndex;
use super::PointIndex;
use super::TetraIndex;

#[cfg(feature = "2d")]
pub type Tetra = super::tetra_2d::Tetra2d;
#[cfg(feature = "2d")]
pub type TetraData = super::tetra_2d::Tetra2dData;

#[cfg(feature = "3d")]
pub type Tetra = super::tetra_3d::Tetra3d;
#[cfg(feature = "3d")]
pub type TetraData = super::tetra_3d::Tetra3dData;

impl Tetra {
    pub fn iter_faces_and_points(&self) -> impl Iterator<Item = (&TetraFace, &PointIndex)> {
        self.iter_faces().zip(self.iter_points())
    }

    pub fn find_face(&self, face: FaceIndex) -> &TetraFace {
        self.iter_faces().find(|f| f.face == face).unwrap()
    }

    pub fn find_face_mut(&mut self, face: FaceIndex) -> &mut TetraFace {
        self.iter_faces_mut().find(|f| f.face == face).unwrap()
    }

    pub fn find_face_opposite(&self, p: PointIndex) -> &TetraFace {
        self.iter_faces_and_points()
            .find(|(_, point)| **point == p)
            .map(|(face, _)| face)
            .unwrap_or_else(|| {
                panic!("find_face_opposite called with point that is not part of the tetra.");
            })
    }

    pub fn find_point_opposite(&self, f: FaceIndex) -> PointIndex {
        self.iter_faces_and_points()
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
