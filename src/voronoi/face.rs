use super::PointIndex;
use super::TetraIndex;

pub struct Face {
    pub p1: PointIndex,
    pub p2: PointIndex,
    pub opposing: Option<OtherTetraInfo>,
}

pub struct OtherTetraInfo {
    pub tetra: TetraIndex,
    pub point: PointIndex,
}
