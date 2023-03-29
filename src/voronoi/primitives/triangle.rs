use super::Float;
use super::Point2d;
use super::Point3d;
use crate::voronoi::delaunay::dimension::DFace;
use crate::voronoi::delaunay::dimension::DFaceData;
use crate::voronoi::delaunay::dimension::DTetra;
use crate::voronoi::delaunay::dimension::DTetraData;
use crate::voronoi::delaunay::face_info::FaceInfo;
use crate::voronoi::math::determinant3x3;
use crate::voronoi::math::solve_system_of_equations;
use crate::voronoi::precision_error::is_negative;
use crate::voronoi::precision_error::is_positive;
use crate::voronoi::precision_error::PrecisionError;
use crate::voronoi::utils::Extent;
use crate::voronoi::PointIndex;
use crate::voronoi::ThreeD;
use crate::voronoi::TwoD;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntersectionType {
    Inside,
    OutsideOneEdge(EdgeIdentifier),
    OutsideTwoEdges(EdgeIdentifier, EdgeIdentifier),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeIdentifier {
    One,
    Two,
    Three,
}

#[derive(Clone, Debug)]
pub struct Triangle {
    pub p1: PointIndex,
    pub p2: PointIndex,
    pub p3: PointIndex,
}

impl DFace for Triangle {
    type Dimension = ThreeD;
    fn points(&self) -> Box<dyn Iterator<Item = PointIndex>> {
        Box::new([self.p1, self.p2, self.p3].into_iter())
    }
}

impl Triangle {
    pub fn get_point_opposite(&self, edge_identifier: EdgeIdentifier) -> PointIndex {
        match edge_identifier {
            EdgeIdentifier::One => self.p1,
            EdgeIdentifier::Two => self.p2,
            EdgeIdentifier::Three => self.p3,
        }
    }

    pub fn get_points_of(&self, edge_identifier: EdgeIdentifier) -> (PointIndex, PointIndex) {
        match edge_identifier {
            EdgeIdentifier::One => (self.p2, self.p3),
            EdgeIdentifier::Two => (self.p3, self.p1),
            EdgeIdentifier::Three => (self.p1, self.p2),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TriangleWithFaces {
    pub p1: PointIndex,
    pub p2: PointIndex,
    pub p3: PointIndex,
    pub f1: FaceInfo,
    pub f2: FaceInfo,
    pub f3: FaceInfo,
}

impl DTetra for TriangleWithFaces {
    type Dimension = TwoD;

    fn faces(&self) -> Box<dyn Iterator<Item = &FaceInfo> + '_> {
        Box::new([&self.f1, &self.f2, &self.f3].into_iter())
    }

    fn faces_mut(&mut self) -> Box<dyn Iterator<Item = &mut FaceInfo> + '_> {
        Box::new([&mut self.f1, &mut self.f2, &mut self.f3].into_iter())
    }

    fn points(&self) -> Box<dyn Iterator<Item = PointIndex> + '_> {
        Box::new([self.p1, self.p2, self.p3].into_iter())
    }

    fn contains_point(&self, p: PointIndex) -> bool {
        self.p1 == p || self.p2 == p || self.p3 == p
    }
}

#[derive(Clone, Debug)]
pub struct TriangleData<P> {
    pub p1: P,
    pub p2: P,
    pub p3: P,
}

impl FromIterator<Point3d> for TriangleData<Point3d> {
    fn from_iter<T: IntoIterator<Item = Point3d>>(points: T) -> Self {
        let mut points = points.into_iter();
        let result = Self {
            p1: points.next().unwrap(),
            p2: points.next().unwrap(),
            p3: points.next().unwrap(),
        };
        assert_eq!(points.next(), None);
        result
    }
}

impl DFaceData for TriangleData<Point3d> {
    type Dimension = ThreeD;
}

impl FromIterator<Point2d> for TriangleData<Point2d> {
    fn from_iter<T: IntoIterator<Item = Point2d>>(points: T) -> Self {
        let mut points = points.into_iter();
        let result = Self {
            p1: points.next().unwrap(),
            p2: points.next().unwrap(),
            p3: points.next().unwrap(),
        };
        assert_eq!(points.next(), None);
        result
    }
}

impl TriangleData<Point2d> {
    fn transform_point_to_canonical_coordinates(&self, point: Point2d) -> (Float, Float) {
        // We solve
        // p = p1 + r (p2 - p1) + s (p3 - p1)
        // where r and s are the coordinates of the point in the (two-dimensional) vector space
        // spanned by the (linearly independent) vectors given by (p2 - p1) and (p3 - p1).
        let a = self.p2 - self.p1;
        let b = self.p3 - self.p1;
        let c = point - self.p1;
        let [r, s] = solve_system_of_equations([[a.x, b.x, c.x], [a.y, b.y, c.y]]);
        (r, s)
    }
}
impl DTetraData for TriangleData<Point2d> {
    type Dimension = TwoD;
    fn all_encompassing<'a>(extent: &Extent<Point2d>) -> Self {
        // An overshooting factor for numerical safety
        let alpha = 1.00;
        let (min, max) = (extent.min, extent.max);
        let p1 = min - (max - min) * alpha;
        let p2 = Point2d::new(min.x, max.y + (max.y - min.y) * (1.0 + alpha));
        let p3 = Point2d::new(max.x + (max.x - min.x) * (1.0 + alpha), min.y);
        Self { p1, p2, p3 }
    }

    fn contains(&self, p: Point2d) -> Result<bool, PrecisionError> {
        let (r, s) = self.transform_point_to_canonical_coordinates(p);
        let values = [r, s, 1.0 - (r + s)];
        let is_definitely_outside = values
            .iter()
            .any(|value| is_negative(*value).unwrap_or(false));
        if is_definitely_outside {
            Ok(false)
        } else {
            for value in values {
                PrecisionError::check(value)?;
            }
            Ok(true)
        }
    }

    fn distance_to_point(&self, p: Point2d) -> Float {
        let distance_to_side = |pa: Point2d, pb: Point2d| {
            ((pb.y - pa.y) * p.x - (pb.x - pa.x) * p.y + pb.x * pa.y - pa.x * pb.y)
                / pa.distance(pb)
        };

        let d1 = distance_to_side(self.p1, self.p2);
        let d2 = distance_to_side(self.p2, self.p3);
        let d3 = distance_to_side(self.p3, self.p1);

        d1.max(d2).max(d3)
    }

    #[rustfmt::skip]
    fn circumcircle_contains(&self, point: Point2d) -> Result<bool, PrecisionError> {
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
    fn is_positively_oriented(&self) -> Result<bool, PrecisionError> {
        is_positive(determinant3x3(
            1.0, self.p1.x, self.p1.y,
            1.0, self.p2.x, self.p2.y,
            1.0, self.p3.x, self.p3.y,
        ))
    }

    fn get_center_of_circumcircle(&self) -> Point2d {
        let a = self.p1;
        let b = self.p2;
        let c = self.p3;
        let d = 2.0 * (a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y));
        Point2d {
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

impl TriangleData<Point3d> {
    pub fn get_line_intersection_type(
        &self,
        q1: Point3d,
        q2: Point3d,
    ) -> Result<IntersectionType, PrecisionError> {
        // We solve the line-triangle intersection equation
        // p1 + r (p2 - p1) + s (p3 - p1) = q1 + t (q2 - q1)
        // for r, s, and t.
        // r and s are the coordinates of the point in the (two-dimensional) vector space
        // spanned by the (linearly independent) vectors given by (p2 - p1) and (p3 - p1).
        let a = self.p2 - self.p1;
        let b = self.p3 - self.p1;
        let k = q2 - q1;
        let c = q1 - self.p1;
        let [r, s, _] = solve_system_of_equations([
            [a.x, b.x, -k.x, c.x],
            [a.y, b.y, -k.y, c.y],
            [a.z, b.z, -k.z, c.z],
        ]);
        self.get_intersection_type(r, s)
    }

    fn get_intersection_type(
        &self,
        r: Float,
        s: Float,
    ) -> Result<IntersectionType, PrecisionError> {
        let identifiers = [
            (is_negative(r)?, EdgeIdentifier::Two),
            (is_negative(s)?, EdgeIdentifier::Three),
            (is_negative(1.0 - (r + s))?, EdgeIdentifier::One),
        ]
        .into_iter()
        .filter(|(state, _)| *state)
        .map(|(_, id)| id)
        .collect::<Vec<_>>();
        Ok(match identifiers.len() {
            0 => IntersectionType::Inside,
            1 => IntersectionType::OutsideOneEdge(identifiers[0]),
            2 => IntersectionType::OutsideTwoEdges(identifiers[0], identifiers[1]),
            _ => panic!("Possibly degenerate case of point lying on one of the edges."),
        })
    }

    pub fn distance_to_point(&self, p: Point3d) -> Float {
        self.closest_point(p).distance(p)
    }

    fn closest_point(&self, p: Point3d) -> Point3d {
        // This is the method employed by embree (https://github.com/embree/embree/blob/master/tutorials/common/math/closest_point.)
        let a = self.p1;
        let b = self.p2;
        let c = self.p3;
        let ab = b - a;
        let ac = c - a;
        let ap = p - a;

        let d1 = ab.dot(ap);
        let d2 = ac.dot(ap);
        if d1 <= 0.0 && d2 <= 0.0 {
            return a;
        };

        let bp = p - b;
        let d3 = ab.dot(bp);
        let d4 = ac.dot(bp);
        if d3 >= 0.0 && d4 <= d3 {
            return b;
        };

        let cp = p - c;
        let d5 = ab.dot(cp);
        let d6 = ac.dot(cp);
        if d6 >= 0.0 && d5 <= d6 {
            return c;
        };

        let vc = d1 * d4 - d3 * d2;
        if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
            let v = d1 / (d1 - d3);
            return a + v * ab;
        }

        let vb = d5 * d2 - d1 * d6;
        if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
            let v = d2 / (d2 - d6);
            return a + v * ac;
        }

        let va = d3 * d6 - d5 * d4;
        if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
            let v = (d4 - d3) / ((d4 - d3) + (d5 - d6));
            return b + v * (c - b);
        }

        let denom = 1.0 / (va + vb + vc);
        let v = vb * denom;
        let w = vc * denom;
        a + v * ab + w * ac
    }
}

#[cfg(test)]
mod tests {
    use super::EdgeIdentifier;
    use super::IntersectionType;
    use super::TriangleData;
    use crate::voronoi::delaunay::dimension::DTetraData;
    use crate::voronoi::precision_error::PrecisionError;
    use crate::voronoi::primitives::Point2d;
    use crate::voronoi::primitives::Point3d;

    fn triangle() -> TriangleData<Point3d> {
        let p1 = Point3d::new(0.0, 0.0, 0.0);
        let p2 = Point3d::new(1.0, 0.0, 0.0);
        let p3 = Point3d::new(0.0, 1.0, 0.0);
        TriangleData::<Point3d> { p1, p2, p3 }
    }

    #[test]
    fn get_intersection_type() {
        let face = triangle();
        let check_two_d_point = |x, y, intersection_type| {
            let q1 = Point3d::new(x, y, -1.0);
            let q2 = Point3d::new(x, y, 1.0);
            let type_ = face.get_line_intersection_type(q1, q2);
            assert_eq!(type_, intersection_type);
        };
        check_two_d_point(0.3, 0.3, Ok(IntersectionType::Inside));
        check_two_d_point(
            -0.1,
            0.3,
            Ok(IntersectionType::OutsideOneEdge(EdgeIdentifier::Two)),
        );
        check_two_d_point(
            0.3,
            -0.1,
            Ok(IntersectionType::OutsideOneEdge(EdgeIdentifier::Three)),
        );
        check_two_d_point(
            0.6,
            0.6,
            Ok(IntersectionType::OutsideOneEdge(EdgeIdentifier::One)),
        );
        check_two_d_point(
            -0.1,
            -0.1,
            Ok(IntersectionType::OutsideTwoEdges(
                EdgeIdentifier::Two,
                EdgeIdentifier::Three,
            )),
        );
    }

    #[test]
    fn two_d_triangle_contains() {
        let triangle = TriangleData::<Point2d> {
            p1: Point2d::new(2.0, 2.0),
            p2: Point2d::new(4.0, 2.0),
            p3: Point2d::new(2.0, 6.0),
        };
        assert_eq!(triangle.contains(Point2d::new(3.0, 3.0)), Ok(true));

        assert_eq!(triangle.contains(Point2d::new(1.0, 1.0)), Ok(false));
        assert_eq!(triangle.contains(Point2d::new(2.0, 9.0)), Ok(false));
        assert_eq!(triangle.contains(Point2d::new(9.0, 2.0)), Ok(false));
        assert_eq!(triangle.contains(Point2d::new(-1.0, 2.0)), Ok(false));

        assert_eq!(
            triangle.contains(Point2d::new(2.0, 2.0)),
            Err(PrecisionError)
        );
        assert_eq!(
            triangle.contains(Point2d::new(4.0, 2.0)),
            Err(PrecisionError)
        );
        assert_eq!(
            triangle.contains(Point2d::new(2.0, 6.0)),
            Err(PrecisionError)
        );

        assert_eq!(
            triangle.contains(Point2d::new(3.0, 2.0)),
            Err(PrecisionError)
        );
        assert_eq!(
            triangle.contains(Point2d::new(2.0, 4.0)),
            Err(PrecisionError)
        );
        assert_eq!(
            triangle.contains(Point2d::new(3.0, 4.0)),
            Err(PrecisionError)
        );
    }
}
