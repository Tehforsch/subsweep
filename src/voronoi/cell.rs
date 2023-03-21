use std::f64::consts::PI;

use super::delaunay::dimension::Dimension;
use super::delaunay::PointIndex;
use super::primitives::Point2d;
use super::primitives::Point3d;
use super::primitives::Vector;
use super::utils::periodic_windows;
use super::CellIndex;
use super::Point;
use super::ThreeD;
use super::TwoD;
use super::VoronoiGrid;
use crate::prelude::Float;

pub trait DimensionCell {
    type Dimension: Dimension;
    fn size(&self) -> Float;
    fn volume(&self) -> Float;
    fn contains(&self, point: Point<Self::Dimension>) -> bool;
    fn iter_neighbours_and_faces<'a>(
        &'a self,
        grid: &'a VoronoiGrid<Self::Dimension>,
    ) -> Box<dyn Iterator<Item = (CellConnection, Float, Point<Self::Dimension>)> + '_>;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CellConnection {
    ToInner(CellIndex),
    ToOuter,
}

pub struct Cell<D: Dimension> {
    pub delaunay_point: PointIndex,
    pub center: Point<D>,
    pub index: CellIndex,
    pub points: Vec<Point<D>>,
    pub connected_cells: Vec<CellConnection>,
}

fn sign(p1: Point2d, p2: Point2d, p3: Point2d) -> Float {
    (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
}

impl Cell<TwoD> {
    pub fn point_windows(&self) -> impl Iterator<Item = (&Point2d, &Point2d)> {
        periodic_windows(&self.points)
    }
}

impl DimensionCell for Cell<TwoD> {
    type Dimension = TwoD;

    fn iter_neighbours_and_faces<'a>(
        &'a self,
        _grid: &'a VoronoiGrid<TwoD>,
    ) -> Box<dyn Iterator<Item = (CellConnection, Float, Point2d)> + '_> {
        Box::new(
            self.connected_cells
                .iter()
                .zip(self.point_windows())
                .map(|(c, (p1, p2))| {
                    let face_area = p1.distance(*p2);
                    let dir = *p1 - *p2;
                    let mut normal = Point2d::new(dir.y, -dir.x).normalize();
                    if (*p1 - self.center).dot(normal) < 0.0 {
                        normal = -normal;
                    }
                    (*c, face_area, normal)
                }),
        )
    }

    fn contains(&self, point: Point2d) -> bool {
        let has_negative = self
            .point_windows()
            .map(|(p1, p2)| sign(point, *p1, *p2))
            .any(|s| s < 0.0);
        let has_positive = self
            .point_windows()
            .map(|(p1, p2)| sign(point, *p1, *p2))
            .any(|s| s > 0.0);

        !(has_negative && has_positive)
    }

    fn size(&self) -> Float {
        (self.volume() / PI).sqrt()
    }

    fn volume(&self) -> Float {
        0.5 * self
            .point_windows()
            .map(|(p1, p2)| p1.x * p2.y - p2.x * p1.y)
            .sum::<Float>()
            .abs()
    }
}

impl DimensionCell for Cell<ThreeD> {
    type Dimension = ThreeD;

    fn contains(&self, _point: Point3d) -> bool {
        todo!()
    }

    fn size(&self) -> Float {
        (3.0 * self.volume() / (4.0 * PI)).cbrt()
    }

    fn volume(&self) -> Float {
        todo!()
    }

    fn iter_neighbours_and_faces<'a>(
        &'a self,
        grid: &'a VoronoiGrid<ThreeD>,
    ) -> Box<dyn Iterator<Item = (CellConnection, Float, Point3d)>> {
        todo!()
    }
}
