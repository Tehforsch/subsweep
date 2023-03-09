use super::super::delaunay::face_info::FaceInfo;
use super::Point3d;
use crate::voronoi::math::determinant4x4;
use crate::voronoi::math::determinant5x5;
use crate::voronoi::precision_error::is_negative;
use crate::voronoi::precision_error::is_positive;
use crate::voronoi::precision_error::PrecisionError;
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

impl Tetrahedron {
    pub fn iter_faces(&self) -> impl Iterator<Item = &FaceInfo> {
        ([&self.f1, &self.f2, &self.f3, &self.f4]).into_iter()
    }

    pub fn iter_points(&self) -> impl Iterator<Item = &PointIndex> {
        ([&self.p1, &self.p2, &self.p3, &self.p4]).into_iter()
    }

    pub fn iter_faces_mut(&mut self) -> impl Iterator<Item = &mut FaceInfo> {
        ([&mut self.f1, &mut self.f2, &mut self.f3, &mut self.f4]).into_iter()
    }
}

impl TetrahedronData {
    pub fn all_encompassing(_points: &[Point3d]) -> TetrahedronData {
        todo!()
    }

    #[rustfmt::skip]
    pub fn contains(&self, point: Point3d) -> Result<bool, PrecisionError> {
        Ok(
               points_are_on_same_side_of_triangle(point, self.p1, (self.p2, self.p3, self.p4))?
            && points_are_on_same_side_of_triangle(point, self.p2, (self.p1, self.p3, self.p4))?
            && points_are_on_same_side_of_triangle(point, self.p3, (self.p1, self.p2, self.p4))?
            && points_are_on_same_side_of_triangle(point, self.p4, (self.p1, self.p2, self.p3))?,
        )
    }

    #[rustfmt::skip]
    pub fn circumcircle_contains(&self, point: Point3d) -> Result<bool, PrecisionError> {
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
    pub fn is_positively_oriented(&self) -> Result<bool, PrecisionError> {
        let determinant = determinant4x4(
            1.0, self.p1.x, self.p1.y, self.p1.z,
            1.0, self.p2.x, self.p2.y, self.p2.z,
            1.0, self.p3.x, self.p3.y, self.p3.z,
            1.0, self.p4.x, self.p4.y, self.p4.z,
        );
        is_positive(determinant)
    }

    pub fn get_center_of_circumcircle(&self) -> Point3d {
        todo!()
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
