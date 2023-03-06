use super::math::determinant4x4;
use super::math::determinant5x5;
use super::tetra::TetraFace;
use super::Point;
use super::PointIndex;

#[derive(Clone)]
pub struct Tetra3d {
    pub p1: PointIndex,
    pub p2: PointIndex,
    pub p3: PointIndex,
    pub p4: PointIndex,
    pub f1: TetraFace,
    pub f2: TetraFace,
    pub f3: TetraFace,
    pub f4: TetraFace,
}

#[derive(Clone)]
pub struct Tetra3dData {
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
    pub p4: Point,
}

impl Tetra3d {
    pub fn iter_faces(&self) -> impl Iterator<Item = &TetraFace> {
        ([&self.f1, &self.f2, &self.f3, &self.f4]).into_iter()
    }

    pub fn iter_points(&self) -> impl Iterator<Item = &PointIndex> {
        ([&self.p1, &self.p2, &self.p3, &self.p4]).into_iter()
    }

    pub fn iter_faces_mut(&mut self) -> impl Iterator<Item = &mut TetraFace> {
        ([&mut self.f1, &mut self.f2, &mut self.f3, &mut self.f4]).into_iter()
    }
}

impl Tetra3dData {
    pub fn all_encompassing(_points: &[Point]) -> Tetra3dData {
        todo!()
    }

    pub fn contains(&self, point: Point) -> bool {
        points_are_on_same_side_of_triangle(point, self.p1, (self.p2, self.p3, self.p4))
            && points_are_on_same_side_of_triangle(point, self.p2, (self.p1, self.p3, self.p4))
            && points_are_on_same_side_of_triangle(point, self.p3, (self.p1, self.p2, self.p4))
            && points_are_on_same_side_of_triangle(point, self.p4, (self.p1, self.p2, self.p3))
    }

    #[rustfmt::skip]
    pub fn circumcircle_contains(&self, point: Point) -> bool {
        // See for example Springel (2009), doi:10.1111/j.1365-2966.2009.15715.x
        debug_assert!(self.is_positively_oriented());
        let a = self.p1;
        let b = self.p2;
        let c = self.p3;
        let d = self.p4;
        let e = point;
        determinant5x5(
            1.0, a.x, a.y, a.z, a.x.powi(2) + a.y.powi(2) + a.z.powi(2),
            1.0, b.x, b.y, b.z, b.x.powi(2) + b.y.powi(2) + b.z.powi(2),
            1.0, c.x, c.y, c.z, c.x.powi(2) + c.y.powi(2) + c.z.powi(2),
            1.0, d.x, d.y, d.z, d.x.powi(2) + d.y.powi(2) + d.z.powi(2),
            1.0, e.x, e.y, e.z, e.x.powi(2) + e.y.powi(2) + e.z.powi(2),
        ) < 0.0
    }

    #[rustfmt::skip]
    pub fn is_positively_oriented(&self) -> bool {
        determinant4x4(
            1.0, self.p1.x, self.p1.y, self.p1.z,
            1.0, self.p2.x, self.p2.y, self.p2.z,
            1.0, self.p3.x, self.p3.y, self.p3.z,
            1.0, self.p4.x, self.p4.y, self.p4.z,
        ) > 0.0
    }

    pub fn get_center_of_circumcircle(&self) -> Point {
        todo!()
    }
}

fn points_are_on_same_side_of_triangle(
    p1: Point,
    p2: Point,
    triangle: (Point, Point, Point),
) -> bool {
    let (p_a, p_b, p_c) = triangle;
    let normal = (p_b - p_a).cross(p_c - p_a);
    let dot_1_sign = (p1 - p_a).dot(normal).signum();
    let dot_2_sign = (p2 - p_a).dot(normal).signum();
    (dot_1_sign * dot_2_sign) >= 0.0
}
