use super::Point2d;
use crate::voronoi::delaunay::dimension::DFace;
use crate::voronoi::delaunay::dimension::DFaceData;
use crate::voronoi::PointIndex;
use crate::voronoi::TwoD;

#[derive(Clone, Debug)]
pub struct Line {
    pub p1: PointIndex,
    pub p2: PointIndex,
}

impl DFace for Line {
    type Dimension = TwoD;
    fn points(&self) -> Box<dyn Iterator<Item = PointIndex>> {
        Box::new([self.p1, self.p2].into_iter())
    }
}

impl Line {
    pub fn contains(&self, point: PointIndex) -> bool {
        self.p1 == point || self.p2 == point
    }

    pub fn other_point(&self, point: PointIndex) -> PointIndex {
        if point == self.p1 {
            self.p2
        } else if point == self.p2 {
            self.p1
        } else {
            panic!("other_point called with point that is not part of the line");
        }
    }
}

#[derive(Clone)]
pub struct LineData<P> {
    pub p1: P,
    pub p2: P,
}

impl DFaceData for LineData<Point2d> {
    type Dimension = TwoD;
}

impl FromIterator<Point2d> for LineData<Point2d> {
    fn from_iter<T: IntoIterator<Item = Point2d>>(points: T) -> Self {
        let mut points = points.into_iter();
        let result = Self {
            p1: points.next().unwrap(),
            p2: points.next().unwrap(),
        };
        assert_eq!(points.next(), None);
        result
    }
}
