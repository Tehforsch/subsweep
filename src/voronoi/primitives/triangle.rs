use std::array::IntoIter;
use std::ops::Add;
use std::ops::Sub;

use num::One;

use super::super::math::traits::Vector2d;
use super::super::math::traits::Vector3d;
use super::Float;
use super::Point2d;
use super::Point3d;
use crate::dimension::ThreeD;
use crate::dimension::TwoD;
use crate::extent::Extent;
use crate::voronoi::delaunay::dimension::DFace;
use crate::voronoi::delaunay::dimension::DFaceData;
use crate::voronoi::delaunay::dimension::DTetra;
use crate::voronoi::delaunay::dimension::DTetraData;
use crate::voronoi::delaunay::face_info::FaceInfo;
use crate::voronoi::delaunay::Point;
use crate::voronoi::math::precision_types::PrecisionError;
use crate::voronoi::math::precision_types::PrecisionPoint2d;
use crate::voronoi::math::precision_types::PrecisionPoint3d;
use crate::voronoi::math::precision_types::TRIANGLE_CONTAINS_EPSILON;
use crate::voronoi::math::precision_types::TRIANGLE_INTERSECTION_TYPE_EPSILON;
use crate::voronoi::math::utils::determinant3x3_sign;
use crate::voronoi::math::utils::solve_system_of_equations;
use crate::voronoi::math::utils::Sign;
use crate::voronoi::PointIndex;

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

    type PointsIter = IntoIter<PointIndex, 3>;

    fn points(&self) -> Self::PointsIter {
        [self.p1, self.p2, self.p3].into_iter()
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

    type PointsIter = IntoIter<PointIndex, 3>;
    type FacesIter<'a> = IntoIter<&'a FaceInfo, 3>;
    type FacesMutIter<'a> = IntoIter<&'a mut FaceInfo, 3>;

    fn points(&self) -> Self::PointsIter {
        [self.p1, self.p2, self.p3].into_iter()
    }

    fn faces(&self) -> Self::FacesIter<'_> {
        [&self.f1, &self.f2, &self.f3].into_iter()
    }

    fn faces_mut(&mut self) -> Self::FacesMutIter<'_> {
        [&mut self.f1, &mut self.f2, &mut self.f3].into_iter()
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

impl<V: Vector2d + Clone + Sub<Output = V> + std::fmt::Debug> TriangleData<V> {
    fn transform_point_to_canonical_coordinates(&self, point: V) -> (V::Float, V::Float) {
        // We solve
        // p = p1 + r (p2 - p1) + s (p3 - p1)
        // where r and s are the coordinates of the point in the (two-dimensional) vector space
        // spanned by the (linearly independent) vectors given by (p2 - p1) and (p3 - p1).
        let a = self.p2.clone() - self.p1.clone();
        let b = self.p3.clone() - self.p1.clone();
        let c = point - self.p1.clone();
        let [r, s] = solve_system_of_equations([[a.x(), b.x(), c.x()], [a.y(), b.y(), c.y()]]);
        (r, s)
    }

    fn generic_contains(&self, p: V) -> Result<bool, PrecisionError> {
        let (r, s) = self.transform_point_to_canonical_coordinates(p);
        let values = [r.clone(), s.clone(), V::Float::one() - (r + s)];
        let signs = || {
            values
                .iter()
                .map(|value| Sign::try_from_val(value, TRIANGLE_CONTAINS_EPSILON))
        };
        let is_definitely_outside = signs().any(|sign| {
            if let Ok(sign) = sign {
                sign.is_negative()
            } else {
                false
            }
        });
        if is_definitely_outside {
            Ok(false)
        } else {
            for sign in signs() {
                sign?.panic_if_zero(|| "Degenerate case of point on edge of triangle");
            }
            Ok(true)
        }
    }
}

impl TriangleData<Point2d> {
    fn arbitrary_precision_contains(&self, p: Point2d) -> bool {
        let lifted = TriangleData {
            p1: PrecisionPoint2d::new(self.p1),
            p2: PrecisionPoint2d::new(self.p2),
            p3: PrecisionPoint2d::new(self.p3),
        };
        lifted.generic_contains(PrecisionPoint2d::new(p)).unwrap()
    }

    fn f64_contains(&self, p: Point2d) -> Result<bool, PrecisionError> {
        self.generic_contains(p)
    }
}

impl DTetraData for TriangleData<Point2d> {
    type Dimension = TwoD;
    fn all_encompassing<'a>(extent: &Extent<Point2d>) -> Self {
        // An overshooting factor for numerical safety
        let alpha = 1.00;
        let (min, max) = (extent.min, extent.max);
        let pa = min - (max - min) * alpha;
        let pb = Point2d::new(min.x, max.y + (max.y - min.y) * (1.0 + alpha));
        let pc = Point2d::new(max.x + (max.x - min.x) * (1.0 + alpha), min.y);
        // Flip pa and pb so that the triangle is positively oriented
        Self {
            p1: pb,
            p2: pa,
            p3: pc,
        }
    }

    fn extent(&self) -> Extent<Point<Self::Dimension>> {
        Extent::from_points([self.p1, self.p2, self.p3].into_iter()).unwrap()
    }

    fn contains(&self, p: Point<Self::Dimension>) -> bool {
        self.f64_contains(p)
            .unwrap_or_else(|_| self.arbitrary_precision_contains(p))
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
    fn circumcircle_contains(&self, point: Point2d) -> bool {
        // See for example Springel (2009), doi:10.1111/j.1365-2966.2009.15715.x
        let a = self.p1;
        let b = self.p2;
        let c = self.p3;
        let d = point;
        let sign = determinant3x3_sign(
            [
                [b.x - a.x, b.y - a.y, (b.x - a.x).powi(2) + (b.y - a.y).powi(2)],
                [c.x - a.x, c.y - a.y, (c.x - a.x).powi(2) + (c.y - a.y).powi(2)],
                [d.x - a.x, d.y - a.y, (d.x - a.x).powi(2) + (d.y - a.y).powi(2)]
            ]
        );
        sign.panic_if_zero(|| "Degenerate case in circumcircle test.").is_negative()
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

impl<V: Vector3d + Clone + Add<Output = V> + Sub<Output = V>> TriangleData<V> {
    pub fn generic_get_line_intersection_type(
        &self,
        q1: V,
        q2: V,
    ) -> Result<IntersectionType, PrecisionError> {
        // We solve the line-triangle intersection equation
        // p1 + r (p2 - p1) + s (p3 - p1) = q1 + t (q2 - q1)
        // for r, s, and t.
        // r and s are the coordinates of the point in the (two-dimensional) vector space
        // spanned by the (linearly independent) vectors given by (p2 - p1) and (p3 - p1).
        let a = self.p2.clone() - self.p1.clone();
        let b = self.p3.clone() - self.p1.clone();
        let k = q2 - q1.clone();
        let c = q1 - self.p1.clone();
        let [r, s, _] = solve_system_of_equations([
            [a.x(), b.x(), -k.x(), c.x()],
            [a.y(), b.y(), -k.y(), c.y()],
            [a.z(), b.z(), -k.z(), c.z()],
        ]);
        self.get_intersection_type(r, s)
    }

    fn get_intersection_type(
        &self,
        r: V::Float,
        s: V::Float,
    ) -> Result<IntersectionType, PrecisionError> {
        let signs: Result<Vec<_>, PrecisionError> =
            [r.clone(), s.clone(), V::Float::one() - (r + s)]
                .into_iter()
                .map(|x| {
                    let sign = Sign::try_from_val(&x, TRIANGLE_INTERSECTION_TYPE_EPSILON);
                    if let Ok(sign) = sign {
                        sign.panic_if_zero(|| {
                            "Degenerate case of point on line (implement 4-to-4 flip)"
                        });
                    }
                    sign
                })
                .collect();
        let identifiers = signs?
            .into_iter()
            .zip([
                EdgeIdentifier::Two,
                EdgeIdentifier::Three,
                EdgeIdentifier::One,
            ])
            .filter(|(sign, _)| sign.is_negative())
            .map(|(_, id)| id)
            .collect::<Vec<_>>();
        Ok(match identifiers.len() {
            0 => IntersectionType::Inside,
            1 => IntersectionType::OutsideOneEdge(identifiers[0]),
            2 => IntersectionType::OutsideTwoEdges(identifiers[0], identifiers[1]),
            _ => panic!("Possibly degenerate case of point lying on one of the edges."),
        })
    }
}

impl TriangleData<Point3d> {
    pub fn get_line_intersection_type(&self, q1: Point3d, q2: Point3d) -> IntersectionType {
        self.generic_get_line_intersection_type(q1, q2)
            .unwrap_or_else(|_| {
                let precision_self = TriangleData {
                    p1: PrecisionPoint3d::new(self.p1),
                    p2: PrecisionPoint3d::new(self.p2),
                    p3: PrecisionPoint3d::new(self.p3),
                };
                precision_self
                    .generic_get_line_intersection_type(
                        PrecisionPoint3d::new(q1),
                        PrecisionPoint3d::new(q2),
                    )
                    .unwrap()
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
        check_two_d_point(0.3, 0.3, IntersectionType::Inside);
        check_two_d_point(
            -0.1,
            0.3,
            IntersectionType::OutsideOneEdge(EdgeIdentifier::Two),
        );
        check_two_d_point(
            0.3,
            -0.1,
            IntersectionType::OutsideOneEdge(EdgeIdentifier::Three),
        );
        check_two_d_point(
            0.6,
            0.6,
            IntersectionType::OutsideOneEdge(EdgeIdentifier::One),
        );
        check_two_d_point(
            -0.1,
            -0.1,
            IntersectionType::OutsideTwoEdges(EdgeIdentifier::Two, EdgeIdentifier::Three),
        );
        std::panic::catch_unwind(|| {
            check_two_d_point(
                0.0,
                0.5,
                IntersectionType::OutsideTwoEdges(EdgeIdentifier::Two, EdgeIdentifier::Three),
            )
        })
        .unwrap_err();
        std::panic::catch_unwind(|| {
            check_two_d_point(
                0.5,
                0.0,
                IntersectionType::OutsideTwoEdges(EdgeIdentifier::Two, EdgeIdentifier::Three),
            )
        })
        .unwrap_err();
    }

    #[test]
    fn two_d_triangle_contains() {
        let triangle = TriangleData::<Point2d> {
            p1: Point2d::new(2.0, 2.0),
            p2: Point2d::new(4.0, 2.0),
            p3: Point2d::new(2.0, 6.0),
        };
        assert!(triangle.contains(Point2d::new(3.0, 3.0)));

        assert!(!triangle.contains(Point2d::new(1.0, 1.0)));
        assert!(!triangle.contains(Point2d::new(2.0, 9.0)));
        assert!(!triangle.contains(Point2d::new(9.0, 2.0)));
        assert!(!triangle.contains(Point2d::new(-1.0, 2.0)));

        let should_panic = |p| {
            std::panic::catch_unwind(|| triangle.contains(p)).unwrap_err();
        };

        should_panic(Point2d::new(2.0, 2.0));
        should_panic(Point2d::new(4.0, 2.0));
        should_panic(Point2d::new(2.0, 6.0));

        should_panic(Point2d::new(3.0, 2.0));
        should_panic(Point2d::new(2.0, 4.0));
        should_panic(Point2d::new(3.0, 4.0));
    }
}
