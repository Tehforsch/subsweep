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
    /// This says whether the two points in the face that FaceIndex
    /// refers to appear in this order in the tetrahedron that this
    /// FaceInfo is part of. For example if a triangle consists of
    /// points (p1, p2, p3) and has a FaceInfo referring to a Face
    /// (p1, p3), flipped should be true since the points appear in
    /// opposite order in the tetrahedron.  If however, the FaceInfo
    /// refers to a Face(p3, p1), flipped should be false.
    pub flipped: bool,
}
