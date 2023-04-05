use super::Float;
use super::Point3d;
use crate::voronoi::utils::periodic_windows_2;

#[derive(Debug)]
pub struct Polygon3d {
    pub points: Vec<Point3d>,
}

impl Polygon3d {
    pub fn area(&self) -> Float {
        let r = self.points[0];
        periodic_windows_2(&self.points)
            .map(|(p1, p2)| 0.5 * (r - *p1).cross(r - *p2).length())
            .sum()
    }
}
