use super::math::solve_system_of_equations;
use super::Point;
use super::PointIndex;
use crate::prelude::Float;
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

    pub fn iter_points(&self) -> impl Iterator<Item = PointIndex> {
        [self.p1, self.p2, self.p3].into_iter()
    }

    pub fn get_other_point(&self, p_a: PointIndex, p_b: PointIndex) -> PointIndex {
        self.iter_points().find(|p| *p != p_a && *p != p_b).unwrap()
    }

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

#[cfg(feature = "3d")]
pub struct FaceData {
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
}

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

#[cfg(feature = "3d")]
impl FaceData {
    pub fn get_line_intersection_type(&self, q1: Point, q2: Point) -> IntersectionType {
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

    fn get_intersection_type(&self, r: Float, s: Float) -> IntersectionType {
        let identifiers = [
            (r < 0.0, EdgeIdentifier::Two),
            (s < 0.0, EdgeIdentifier::Three),
            (r + s > 1.0, EdgeIdentifier::One),
        ]
        .into_iter()
        .filter(|(state, _)| *state)
        .map(|(_, id)| id)
        .collect::<Vec<_>>();
        match identifiers.len() {
            0 => IntersectionType::Inside,
            1 => IntersectionType::OutsideOneEdge(identifiers[0]),
            2 => IntersectionType::OutsideTwoEdges(identifiers[0], identifiers[1]),
            _ => panic!("Possibly degenerate case of point lying on one of the edges."),
        }
    }
}

#[cfg(all(test, feature = "3d"))]
mod tests {
    use super::FaceData;
    use crate::voronoi::face::EdgeIdentifier;
    use crate::voronoi::face::IntersectionType;
    use crate::voronoi::Point;

    fn triangle() -> FaceData {
        let p1 = Point::new(0.0, 0.0, 0.0);
        let p2 = Point::new(1.0, 0.0, 0.0);
        let p3 = Point::new(0.0, 1.0, 0.0);
        FaceData { p1, p2, p3 }
    }

    #[test]
    fn get_intersection_type() {
        let face = triangle();
        let check_two_d_point = |x, y, intersection_type| {
            let q1 = Point::new(x, y, -1.0);
            let q2 = Point::new(x, y, 1.0);
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
    }
}
