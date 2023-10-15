use super::FaceIndex;
use super::PointIndex;
use super::TetraIndex;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConnectionData {
    pub tetra: TetraIndex,
    pub point: PointIndex,
}

#[derive(Debug, Clone, Copy)]
pub struct FaceInfo {
    pub face: FaceIndex,
    pub opposing: Option<ConnectionData>,
    /// Whether the normal of the face points into the tetrahedron or not.
    pub flipped: bool,
}

impl FaceInfo {
    pub fn flipped(self) -> Self {
        Self {
            face: self.face,
            opposing: self.opposing,
            flipped: !self.flipped,
        }
    }
}
