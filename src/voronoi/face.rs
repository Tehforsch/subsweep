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

    pub fn get_other_point(&self, point: PointIndex) -> PointIndex {
        if point == self.p1 {
            self.p2
        } else if point == self.p2 {
            self.p1
        } else {
            panic!("Point not in face: {:?}", point)
        }
    }
}
