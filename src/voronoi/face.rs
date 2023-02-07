use super::PointIndex;

#[derive(Debug)]
pub struct Face {
    pub p1: PointIndex,
    pub p2: PointIndex,
}

impl Face {
    pub fn contains_point(&self, point: PointIndex) -> bool {
        self.p1 == point || self.p2 == point
    }
}
