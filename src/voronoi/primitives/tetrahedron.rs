use std::array::IntoIter;
use std::ops::Sub;

use num::FromPrimitive;
use num::One;
use num::ToPrimitive;

use super::super::delaunay::face_info::FaceInfo;
use super::super::math::traits::Dot;
use super::super::math::traits::Vector3d;
use super::triangle::TriangleData;
use super::Float;
use super::Point3d;
use crate::dimension::ThreeD;
use crate::extent::Extent;
use crate::voronoi::delaunay::dimension::DTetra;
use crate::voronoi::delaunay::dimension::DTetraData;
use crate::voronoi::delaunay::Point;
use crate::voronoi::math::precision_types::PrecisionError;
use crate::voronoi::math::precision_types::PrecisionFloat;
use crate::voronoi::math::precision_types::PrecisionPoint3d;
use crate::voronoi::math::precision_types::TETRAHEDRON_POINTS_ON_SAME_SIDE_EPSILON;
use crate::voronoi::math::traits::Cross3d;
use crate::voronoi::math::utils::determinant4x4;
use crate::voronoi::math::utils::determinant5x5;
use crate::voronoi::math::utils::lift_matrix;
use crate::voronoi::math::utils::solve_3x4_system_of_equations_error;
use crate::voronoi::math::utils::Sign;
use crate::voronoi::PointIndex;

#[derive(Clone, Debug)]
pub struct Tetrahedron {
    pub p1: PointIndex,
    pub p2: PointIndex,
    pub p3: PointIndex,
    pub p4: PointIndex,
    pub f1: FaceInfo,
    pub f2: FaceInfo,
    pub f3: FaceInfo,
    pub f4: FaceInfo,
}

#[derive(Clone, Debug)]
pub struct TetrahedronData {
    pub p1: Point3d,
    pub p2: Point3d,
    pub p3: Point3d,
    pub p4: Point3d,
}

impl DTetra for Tetrahedron {
    type Dimension = ThreeD;

    type PointsIter = IntoIter<PointIndex, 4>;
    type FacesIter<'a> = IntoIter<&'a FaceInfo, 4>;
    type FacesMutIter<'a> = IntoIter<&'a mut FaceInfo, 4>;

    fn points(&self) -> Self::PointsIter {
        [self.p1, self.p2, self.p3, self.p4].into_iter()
    }

    fn faces(&self) -> Self::FacesIter<'_> {
        [&self.f1, &self.f2, &self.f3, &self.f4].into_iter()
    }

    fn faces_mut(&mut self) -> Self::FacesMutIter<'_> {
        [&mut self.f1, &mut self.f2, &mut self.f3, &mut self.f4].into_iter()
    }

    fn contains_point(&self, p: PointIndex) -> bool {
        self.p1 == p || self.p2 == p || self.p3 == p || self.p4 == p
    }
}

impl FromIterator<Point3d> for TetrahedronData {
    fn from_iter<T: IntoIterator<Item = Point3d>>(points: T) -> Self {
        let mut points = points.into_iter();
        let result = Self {
            p1: points.next().unwrap(),
            p2: points.next().unwrap(),
            p3: points.next().unwrap(),
            p4: points.next().unwrap(),
        };
        assert_eq!(points.next(), None);
        result
    }
}

impl TetrahedronData {
    fn f64_contains(&self, point: Point3d) -> Result<bool, PrecisionError> {
        Ok(
            points_are_on_same_side_of_triangle(point, self.p1, (self.p2, self.p3, self.p4))?
                && points_are_on_same_side_of_triangle(
                    point,
                    self.p2,
                    (self.p1, self.p3, self.p4),
                )?
                && points_are_on_same_side_of_triangle(
                    point,
                    self.p3,
                    (self.p1, self.p2, self.p4),
                )?
                && points_are_on_same_side_of_triangle(
                    point,
                    self.p4,
                    (self.p1, self.p2, self.p3),
                )?,
        )
    }

    fn arbitrary_precision_contains(&self, point: Point3d) -> bool {
        let p1 = PrecisionPoint3d::new(self.p1);
        let p2 = PrecisionPoint3d::new(self.p2);
        let p3 = PrecisionPoint3d::new(self.p3);
        let p4 = PrecisionPoint3d::new(self.p4);
        let point = PrecisionPoint3d::new(point);
        points_are_on_same_side_of_triangle(
            point.clone(),
            p1.clone(),
            (p2.clone(), p3.clone(), p4.clone()),
        )
        .unwrap()
            && points_are_on_same_side_of_triangle(
                point.clone(),
                p2.clone(),
                (p1.clone(), p3.clone(), p4.clone()),
            )
            .unwrap()
            && points_are_on_same_side_of_triangle(
                point.clone(),
                p3.clone(),
                (p1.clone(), p2.clone(), p4.clone()),
            )
            .unwrap()
            && points_are_on_same_side_of_triangle(point, p4, (p1, p2, p3)).unwrap()
    }
}

impl DTetraData for TetrahedronData {
    type Dimension = ThreeD;

    fn all_encompassing<'a>(extent: &Extent<Point3d>) -> Self {
        // An overshooting factor for numerical safety
        let alpha = 0.01;
        let dir = extent.max - extent.min;
        let projected = extent.max + dir * (2.1 + alpha);
        let p1 = extent.min - dir * alpha;
        let p2 = Point3d::new(projected.x, extent.min.y, extent.min.z);
        let p3 = Point3d::new(extent.min.x, projected.y, extent.min.z);
        let p4 = Point3d::new(extent.min.x, extent.min.y, projected.z);
        Self { p1, p2, p3, p4 }
    }

    fn extent(&self) -> Extent<Point<Self::Dimension>> {
        Extent::from_points([self.p1, self.p2, self.p3, self.p4].into_iter()).unwrap()
    }

    #[rustfmt::skip]
    fn contains(&self, point: Point3d) -> bool {
        self.f64_contains(point).unwrap_or_else(|_| self.arbitrary_precision_contains(point))
    }

    /// This only works if the point is outside of the tetrahedron
    fn distance_to_point(&self, p: Point3d) -> Float {
        if self
            .f64_contains(p)
            .unwrap_or_else(|_| self.arbitrary_precision_contains(p))
        {
            return 0.0;
        }
        let a1 = TriangleData {
            p1: self.p1,
            p2: self.p2,
            p3: self.p3,
        }
        .distance_to_point(p);
        let a2 = TriangleData {
            p1: self.p2,
            p2: self.p3,
            p3: self.p4,
        }
        .distance_to_point(p);
        let a3 = TriangleData {
            p1: self.p3,
            p2: self.p4,
            p3: self.p1,
        }
        .distance_to_point(p);
        let a4 = TriangleData {
            p1: self.p4,
            p2: self.p1,
            p3: self.p2,
        }
        .distance_to_point(p);
        a1.min(a2).min(a3).min(a4)
    }

    fn circumcircle_contains(&self, point: Point3d) -> bool {
        self.circumcircle_contains_float(point).unwrap_or_else(|_| {
            let a = self.p1;
            let b = self.p2;
            let c = self.p3;
            let d = self.p4;
            let e = point;
            let matrix = [
                [1.0, a.x, a.y, a.z, a.x.powi(2) + a.y.powi(2) + a.z.powi(2)],
                [1.0, b.x, b.y, b.z, b.x.powi(2) + b.y.powi(2) + b.z.powi(2)],
                [1.0, c.x, c.y, c.z, c.x.powi(2) + c.y.powi(2) + c.z.powi(2)],
                [1.0, d.x, d.y, d.z, d.x.powi(2) + d.y.powi(2) + d.z.powi(2)],
                [1.0, e.x, e.y, e.z, e.x.powi(2) + e.y.powi(2) + e.z.powi(2)],
            ];
            Sign::of(determinant5x5(lift_matrix(matrix)))
                .panic_if_zero(|| {
                    format!(
                        "Degenerate case in circumcircle test of tetrahedron: {:?}. {:?}",
                        self, matrix
                    )
                })
                .is_negative()
        })
    }

    fn get_center_of_circumcircle(&self) -> Point3d {
        self.get_center_of_circumcircle_float().unwrap_or_else(|_| {
            let p1 = PrecisionPoint3d::new(self.p1);
            let p2 = PrecisionPoint3d::new(self.p2);
            let p3 = PrecisionPoint3d::new(self.p3);
            let p4 = PrecisionPoint3d::new(self.p4);

            let v1 = p1.x.pow(2) + p1.y.pow(2) + p1.z.pow(2);
            let v2 = p2.x.pow(2) + p2.y.pow(2) + p2.z.pow(2);
            let v3 = p3.x.pow(2) + p3.y.pow(2) + p3.z.pow(2);
            let v4 = p4.x.pow(2) + p4.y.pow(2) + p4.z.pow(2);
            let one = || PrecisionFloat::one();
            let dx = determinant4x4([
                [v1.clone(), p1.y.clone(), p1.z.clone(), one()],
                [v2.clone(), p2.y.clone(), p2.z.clone(), one()],
                [v3.clone(), p3.y.clone(), p3.z.clone(), one()],
                [v4.clone(), p4.y.clone(), p4.z.clone(), one()],
            ]);
            let dy = -determinant4x4([
                [v1.clone(), p1.x.clone(), p1.z.clone(), one()],
                [v2.clone(), p2.x.clone(), p2.z.clone(), one()],
                [v3.clone(), p3.x.clone(), p3.z.clone(), one()],
                [v4.clone(), p4.x.clone(), p4.z.clone(), one()],
            ]);
            let dz = determinant4x4([
                [v1, p1.x.clone(), p1.y.clone(), one()],
                [v2, p2.x.clone(), p2.y.clone(), one()],
                [v3, p3.x.clone(), p3.y.clone(), one()],
                [v4, p4.x.clone(), p4.y.clone(), one()],
            ]);
            let a = determinant4x4([
                [p1.x, p1.y, p1.z, one()],
                [p2.x, p2.y, p2.z, one()],
                [p3.x, p3.y, p3.z, one()],
                [p4.x, p4.y, p4.z, one()],
            ]);
            let two = PrecisionFloat::from_f64(2.0).unwrap();
            let factor = one() / (two * a);
            Point3d::new(
                (dx * factor.clone()).to_f64().unwrap(),
                (dy * factor.clone()).to_f64().unwrap(),
                (dz * factor).to_f64().unwrap(),
            )
        })
    }
}

impl TetrahedronData {
    fn circumcircle_contains_float(&self, point: Point3d) -> Result<bool, PrecisionError> {
        // Taken from Arepo - InSphere_Errorbound
        let ax = self.p1.x - point.x;
        let ay = self.p1.y - point.y;
        let az = self.p1.z - point.z;

        let bx = self.p2.x - point.x;
        let by = self.p2.y - point.y;
        let bz = self.p2.z - point.z;

        let cx = self.p3.x - point.x;
        let cy = self.p3.y - point.y;
        let cz = self.p3.z - point.z;

        let dx = self.p4.x - point.x;
        let dy = self.p4.y - point.y;
        let dz = self.p4.z - point.z;

        let axby = ax * by;
        let bxay = bx * ay;
        let bxcy = bx * cy;
        let cxby = cx * by;
        let cxdy = cx * dy;
        let dxcy = dx * cy;
        let dxay = dx * ay;
        let axdy = ax * dy;
        let axcy = ax * cy;
        let cxay = cx * ay;
        let bxdy = bx * dy;
        let dxby = dx * by;

        let ab = axby - bxay;
        let bc = bxcy - cxby;
        let cd = cxdy - dxcy;
        let da = dxay - axdy;
        let ac = axcy - cxay;
        let bd = bxdy - dxby;

        let abc = az * bc - bz * ac + cz * ab;
        let bcd = bz * cd - cz * bd + dz * bc;
        let cda = cz * da + dz * ac + az * cd;
        let dab = dz * ab + az * bd + bz * da;

        let a2 = ax * ax + ay * ay + az * az;
        let b2 = bx * bx + by * by + bz * bz;
        let c2 = cx * cx + cy * cy + cz * cz;
        let d2 = dx * dx + dy * dy + dz * dz;

        let x = (c2 * dab - d2 * abc) + (a2 * bcd - b2 * cda);

        let ab = axby.abs() + bxay.abs();
        let bc = bxcy.abs() + cxby.abs();
        let cd = cxdy.abs() + dxcy.abs();
        let da = dxay.abs() + axdy.abs();
        let ac = axcy.abs() + cxay.abs();
        let bd = bxdy.abs() + dxby.abs();

        let az = az.abs();
        let bz = bz.abs();
        let cz = cz.abs();
        let dz = dz.abs();

        let abc = az * bc + bz * ac + cz * ab;
        let bcd = bz * cd + cz * bd + dz * bc;
        let cda = cz * da + dz * ac + az * cd;
        let dab = dz * ab + az * bd + bz * da;

        let size_limit = (c2 * dab + d2 * abc) + (a2 * bcd + b2 * cda);
        let error_bound = 1.0e-14 * size_limit;
        Ok(Sign::try_from_val(&x, error_bound)?.is_positive())
    }

    #[rustfmt::skip]
    fn get_center_of_circumcircle_float(&self) -> Result<Point3d, PrecisionError> {
        let p0 = self.p1;
        let p1 = self.p2;
        let p2 = self.p3;
        let p3 = self.p4;
        // Taken from Arepo - update_circumcircle
        let ax = p1.x - p0.x;
        let ay = p1.y - p0.y;
        let az = p1.z - p0.z;

        let bx = p2.x - p0.x;
        let by = p2.y - p0.y;
        let bz = p2.z - p0.z;

        let cx = p3.x - p0.x;
        let cy = p3.y - p0.y;
        let cz = p3.z - p0.z;

        let x = Point3d::from(solve_3x4_system_of_equations_error([
            [p1.x - p0.x, p1.y - p0.y, p1.z - p0.z, 0.5 * (ax * ax + ay * ay + az * az)],
            [p2.x - p0.x, p2.y - p0.y, p2.z - p0.z, 0.5 * (bx * bx + by * by + bz * bz)],
            [p3.x - p0.x, p3.y - p0.y, p3.z - p0.z, 0.5 * (cx * cx + cy * cy + cz * cz)],
        ])?);
        Ok(x + p0)
    }
}

fn points_are_on_same_side_of_triangle<P: Vector3d + Cross3d + Sub<Output = P> + Dot + Clone>(
    p1: P,
    p2: P,
    triangle: (P, P, P),
) -> Result<bool, PrecisionError> {
    let (p_a, p_b, p_c) = triangle;
    let normal = (p_b - p_a.clone()).cross(&(p_c - p_a.clone()));
    let dot_1_sign = Sign::try_from_val(
        &(p1 - p_a.clone()).dot(normal.clone()),
        TETRAHEDRON_POINTS_ON_SAME_SIDE_EPSILON,
    )?;
    let dot_2_sign = Sign::try_from_val(
        &(p2 - p_a).dot(normal),
        TETRAHEDRON_POINTS_ON_SAME_SIDE_EPSILON,
    )?;
    Ok((dot_1_sign * dot_2_sign)
        .panic_if_zero(|| "Degenerate case: point on line of triangle.")
        .is_positive())
}

#[cfg(test)]
mod tests {
    use super::super::Point3d;
    use super::TetrahedronData;
    use crate::test_utils::assert_float_is_close;
    use crate::voronoi::delaunay::dimension::DTetraData;

    #[test]
    fn center_of_circumsphere() {
        let tetra = TetrahedronData {
            p1: Point3d::new(0.0, 0.0, 0.0),
            p2: Point3d::new(1.0, 0.123, 0.456),
            p3: Point3d::new(0.456, 1.0, 0.123),
            p4: Point3d::new(0.123, 0.456, 1.0),
        };
        let circumsphere_center = tetra.get_center_of_circumcircle();
        let d = tetra.p1.distance(circumsphere_center);
        assert_float_is_close(d, tetra.p2.distance(circumsphere_center));
        assert_float_is_close(d, tetra.p3.distance(circumsphere_center));
        assert_float_is_close(d, tetra.p4.distance(circumsphere_center));
    }

    #[test]
    fn circumcircle_contains_precision() {
        let tetra = TetrahedronData {
            p1: Point3d::new(0.19236904054484075, 0.1937984500812517, 0.18486863500429718),
            p2: Point3d::new(0.192360756554688, 0.19400002939544703, 0.18517853170782586),
            p3: Point3d::new(0.19232935691504563, 0.1938108287106263, 0.18504746257365087),
            p4: Point3d::new(0.1926691416553181, 0.1940969382421931, 0.18488871435170906),
        };
        let p = Point3d::new(
            0.19263782374657437,
            0.19383462451181724,
            0.18508570960956586,
        );
        assert!(!tetra.circumcircle_contains(p));
    }
}
