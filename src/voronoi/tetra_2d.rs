use super::math::determinant3x3;
use super::tetra::TetraFace;
use super::Point;
use super::PointIndex;
use crate::voronoi::precision_error::is_negative;
use crate::voronoi::precision_error::is_positive;
use crate::voronoi::precision_error::PrecisionError;
use crate::voronoi::utils::sign;

#[derive(Clone, Debug)]
pub struct Tetra2d {
    pub p1: PointIndex,
    pub p2: PointIndex,
    pub p3: PointIndex,
    pub f1: TetraFace,
    pub f2: TetraFace,
    pub f3: TetraFace,
}

impl Tetra2d {
    pub fn iter_faces(&self) -> impl Iterator<Item = &TetraFace> {
        ([&self.f1, &self.f2, &self.f3]).into_iter()
    }

    pub fn iter_points(&self) -> impl Iterator<Item = &PointIndex> {
        ([&self.p1, &self.p2, &self.p3]).into_iter()
    }

    pub fn iter_faces_mut(&mut self) -> impl Iterator<Item = &mut TetraFace> {
        ([&mut self.f1, &mut self.f2, &mut self.f3]).into_iter()
    }
}

#[derive(Debug)]
pub struct Tetra2dData {
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
}

impl Tetra2dData {
    pub fn all_encompassing(points: &[Point]) -> Self {
        let (min, max) = get_min_and_max(points).unwrap();
        assert!(
            (max - min).min_element() > 0.0,
            "Could not construct encompassing tetra for points (zero extent along one axis)"
        );
        // An overshooting factor for numerical safety
        let alpha = 1.00;
        let p1 = min - (max - min) * alpha;
        let p2 = Point::new(min.x, max.y + (max.y - min.y) * (1.0 + alpha));
        let p3 = Point::new(max.x + (max.x - min.x) * (1.0 + alpha), min.y);
        Self { p1, p2, p3 }
    }

    pub fn contains(&self, point: Point) -> Result<bool, PrecisionError> {
        let d1 = sign(point, self.p1, self.p2);
        let d2 = sign(point, self.p2, self.p3);
        let d3 = sign(point, self.p3, self.p1);

        let has_neg = is_negative(d1)? || is_negative(d2)? || is_negative(d3)?;
        let has_pos = is_positive(d1)? || is_positive(d2)? || is_positive(d3)?;

        Ok(!(has_neg && has_pos))
    }

    #[rustfmt::skip]
    pub fn circumcircle_contains(&self, point: Point) -> Result<bool, PrecisionError> {
        // See for example Springel (2009), doi:10.1111/j.1365-2966.2009.15715.x
        debug_assert!(self.is_positively_oriented().unwrap());
        let a = self.p1;
        let b = self.p2;
        let c = self.p3;
        let d = point;
        is_negative(determinant3x3(
            b.x - a.x, b.y - a.y, (b.x - a.x).powi(2) + (b.y - a.y).powi(2),
            c.x - a.x, c.y - a.y, (c.x - a.x).powi(2) + (c.y - a.y).powi(2),
            d.x - a.x, d.y - a.y, (d.x - a.x).powi(2) + (d.y - a.y).powi(2)
        ))
    }

    #[rustfmt::skip]
    pub fn is_positively_oriented(&self) -> Result<bool, PrecisionError> {
        is_positive(determinant3x3(
            1.0, self.p1.x, self.p1.y,
            1.0, self.p2.x, self.p2.y,
            1.0, self.p3.x, self.p3.y,
        ))
    }

    pub fn get_center_of_circumcircle(&self) -> Point {
        let a = self.p1;
        let b = self.p2;
        let c = self.p3;
        let d = 2.0 * (a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y));
        Point {
            x: 1.0 / d
                * ((a.x.powi(2) + a.y.powi(2)) * (b.y - c.y)
                    + (b.x.powi(2) + b.y.powi(2)) * (c.y - a.y)
                    + (c.x.powi(2) + c.y.powi(2)) * (a.y - b.y)),
            y: 1.0 / d
                * ((a.x.powi(2) + a.y.powi(2)) * (c.x - b.x)
                    + (b.x.powi(2) + b.y.powi(2)) * (a.x - c.x)
                    + (c.x.powi(2) + c.y.powi(2)) * (b.x - a.x)),
        }
    }
}

fn get_min_and_max(points: &[Point]) -> Option<(Point, Point)> {
    let mut min = None;
    let mut max = None;
    let update_min = |min: &mut Option<Point>, pos: Point| {
        if let Some(ref mut min) = min {
            *min = min.min(pos);
        } else {
            *min = Some(pos);
        }
    };
    let update_max = |max: &mut Option<Point>, pos: Point| {
        if let Some(ref mut max) = max {
            *max = max.max(pos);
        } else {
            *max = Some(pos);
        }
    };
    for p in points {
        update_min(&mut min, *p);
        update_max(&mut max, *p);
    }
    Some((min?, max?))
}
