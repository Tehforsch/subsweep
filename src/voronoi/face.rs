use super::PointIndex;

#[derive(Clone, Debug)]
pub struct Face {
    pub p1: PointIndex,
    pub p2: PointIndex,
    #[cfg(feature = "3d")]
    pub p3: PointIndex,
}

#[cfg(feature = "2d")]
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

#[cfg(feature = "3d")]
impl Face {
    pub fn contains_point(&self, point: PointIndex) -> bool {
        self.p1 == point || self.p2 == point || self.p3 == point
    }
}
