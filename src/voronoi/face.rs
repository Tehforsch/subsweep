use generational_arena::Index;

pub struct Face {
    pub p1: Index,
    pub p2: Index,
    pub opposing: Option<OtherTetraInfo>,
}

pub struct OtherTetraInfo {
    pub tetra: Index,
    pub point: Index,
}
