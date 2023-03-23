use super::super::delaunay::face_info::FaceInfo;
use super::Float;
use super::Point3d;
use crate::voronoi::delaunay::dimension::DimensionTetra;
use crate::voronoi::delaunay::dimension::DimensionTetraData;
use crate::voronoi::math::determinant4x4;
use crate::voronoi::math::determinant5x5;
use crate::voronoi::precision_error::is_negative;
use crate::voronoi::precision_error::is_positive;
use crate::voronoi::precision_error::PrecisionError;
use crate::voronoi::utils::min_and_max;
use crate::voronoi::PointIndex;
use crate::voronoi::ThreeD;

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

impl DimensionTetra for Tetrahedron {
    type Dimension = ThreeD;

    fn faces(&self) -> Box<dyn Iterator<Item = &FaceInfo> + '_> {
        Box::new([&self.f1, &self.f2, &self.f3, &self.f4].into_iter())
    }

    fn faces_mut(&mut self) -> Box<dyn Iterator<Item = &mut FaceInfo> + '_> {
        Box::new([&mut self.f1, &mut self.f2, &mut self.f3, &mut self.f4].into_iter())
    }

    fn points(&self) -> Box<dyn Iterator<Item = PointIndex> + '_> {
        Box::new([self.p1, self.p2, self.p3, self.p4].into_iter())
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

impl DimensionTetraData for TetrahedronData {
    type Dimension = ThreeD;

    fn all_encompassing<'a>(points: Box<dyn Iterator<Item = Point3d> + 'a>) -> Self {
        let (min, max) = min_and_max(points).unwrap();
        assert!(
            (max - min).min_element() > 0.0,
            "Could not construct encompassing tetra for points (zero extent along one axis)"
        );
        // An overshooting factor for numerical safety
        let alpha = 0.01;
        let dir = max - min;
        let projected = max + dir * (3.0 + alpha);
        let p1 = min - dir * alpha;
        let p2 = Point3d::new(projected.x, min.y, min.z);
        let p3 = Point3d::new(min.x, projected.y, min.z);
        let p4 = Point3d::new(min.x, min.y, projected.z);
        Self { p1, p2, p3, p4 }
    }

    #[rustfmt::skip]
    fn contains(&self, point: Point3d) -> Result<bool, PrecisionError> {
        Ok(
               points_are_on_same_side_of_triangle(point, self.p1, (self.p2, self.p3, self.p4))?
            && points_are_on_same_side_of_triangle(point, self.p2, (self.p1, self.p3, self.p4))?
            && points_are_on_same_side_of_triangle(point, self.p3, (self.p1, self.p2, self.p4))?
            && points_are_on_same_side_of_triangle(point, self.p4, (self.p1, self.p2, self.p3))?,
        )
    }

    fn distance_to_point(&self, p: Point3d) -> Float {
        todo!()
    }

    #[rustfmt::skip]
    fn circumcircle_contains(&self, point: Point3d) -> Result<bool, PrecisionError> {
        // See for example Springel (2009), doi:10.1111/j.1365-2966.2009.15715.x
        debug_assert!(self.is_positively_oriented().unwrap());
        let a = self.p1;
        let b = self.p2;
        let c = self.p3;
        let d = self.p4;
        let e = point;
        is_negative(determinant5x5(
            1.0, a.x, a.y, a.z, a.x.powi(2) + a.y.powi(2) + a.z.powi(2),
            1.0, b.x, b.y, b.z, b.x.powi(2) + b.y.powi(2) + b.z.powi(2),
            1.0, c.x, c.y, c.z, c.x.powi(2) + c.y.powi(2) + c.z.powi(2),
            1.0, d.x, d.y, d.z, d.x.powi(2) + d.y.powi(2) + d.z.powi(2),
            1.0, e.x, e.y, e.z, e.x.powi(2) + e.y.powi(2) + e.z.powi(2),
        ))
    }

    #[rustfmt::skip]
    fn is_positively_oriented(&self) -> Result<bool, PrecisionError> {
        let determinant = determinant4x4(
            1.0, self.p1.x, self.p1.y, self.p1.z,
            1.0, self.p2.x, self.p2.y, self.p2.z,
            1.0, self.p3.x, self.p3.y, self.p3.z,
            1.0, self.p4.x, self.p4.y, self.p4.z,
        );
        is_positive(determinant)
    }

    #[rustfmt::skip]
    fn get_center_of_circumcircle(&self) -> Point3d {
        let v1 = self.p1.x.powi(2) + self.p1.y.powi(2) + self.p1.z.powi(2);
        let v2 = self.p2.x.powi(2) + self.p2.y.powi(2) + self.p2.z.powi(2);
        let v3 = self.p3.x.powi(2) + self.p3.y.powi(2) + self.p3.z.powi(2);
        let v4 = self.p4.x.powi(2) + self.p4.y.powi(2) + self.p4.z.powi(2);
        let dx = determinant4x4(
            v1, self.p1.y, self.p1.z, 1.0,
            v2, self.p2.y, self.p2.z, 1.0,
            v3, self.p3.y, self.p3.z, 1.0,
            v4, self.p4.y, self.p4.z, 1.0,
        );
        let dy = -determinant4x4(
            v1, self.p1.x, self.p1.z, 1.0,
            v2, self.p2.x, self.p2.z, 1.0,
            v3, self.p3.x, self.p3.z, 1.0,
            v4, self.p4.x, self.p4.z, 1.0,
        );
        let dz = determinant4x4(
            v1, self.p1.x, self.p1.y, 1.0,
            v2, self.p2.x, self.p2.y, 1.0,
            v3, self.p3.x, self.p3.y, 1.0,
            v4, self.p4.x, self.p4.y, 1.0,
        );
        let a = determinant4x4(
            self.p1.x, self.p1.y, self.p1.z, 1.0,
            self.p2.x, self.p2.y, self.p2.z, 1.0,
            self.p3.x, self.p3.y, self.p3.z, 1.0,
            self.p4.x, self.p4.y, self.p4.z, 1.0,
        );
        Point3d::new(dx,dy,dz) / (2.0 * a)
    }
}

fn points_are_on_same_side_of_triangle(
    p1: Point3d,
    p2: Point3d,
    triangle: (Point3d, Point3d, Point3d),
) -> Result<bool, PrecisionError> {
    let (p_a, p_b, p_c) = triangle;
    let normal = (p_b - p_a).cross(p_c - p_a);
    let dot_1_sign = (p1 - p_a).dot(normal).signum();
    let dot_2_sign = (p2 - p_a).dot(normal).signum();
    is_positive(dot_1_sign * dot_2_sign)
}

#[cfg(test)]
mod tests {
    use super::super::Point3d;
    use super::TetrahedronData;
    use crate::test_utils::assert_float_is_close;
    use crate::voronoi::delaunay::dimension::DimensionTetraData;

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
}
