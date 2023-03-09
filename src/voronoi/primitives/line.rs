use super::Point2d;
use crate::voronoi::delaunay::dimension::DimensionFace;
use crate::voronoi::delaunay::dimension::DimensionFaceData;
use crate::voronoi::PointIndex;
use crate::voronoi::TwoD;

#[derive(Clone, Debug)]
pub struct Line {
    pub p1: PointIndex,
    pub p2: PointIndex,
}

impl DimensionFace for Line {
    type Dimension = TwoD;
    fn points(&self) -> Box<dyn Iterator<Item = PointIndex>> {
        Box::new([self.p1, self.p2].into_iter())
    }
}

#[derive(Clone)]
pub struct LineData<P> {
    pub p1: P,
    pub p2: P,
}

impl DimensionFaceData for LineData<Point2d> {
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
