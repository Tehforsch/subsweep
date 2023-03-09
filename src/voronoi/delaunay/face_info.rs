use super::FaceIndex;
use super::PointIndex;
use super::TetraIndex;

#[derive(Debug, Clone, Copy)]
pub struct ConnectionData {
    pub tetra: TetraIndex,
    pub point: PointIndex,
}

#[derive(Debug, Clone, Copy)]
pub struct FaceInfo {
    pub face: FaceIndex,
    pub opposing: Option<ConnectionData>,
}
