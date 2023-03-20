use std::f64::consts::PI;

use bevy::utils::StableHashMap;

use super::delaunay::dimension::Dimension;
use super::delaunay::dimension::DimensionTetra;
use super::delaunay::dimension::DimensionTetraData;
use super::delaunay::Delaunay;
use super::delaunay::DelaunayTriangulation;
use super::delaunay::PointIndex;
use super::delaunay::TetraIndex;
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
use crate::voronoi::delaunay::dimension::DimensionFace;

pub trait DimensionCell {
    type Dimension: Dimension;
    fn size(&self) -> Float;
    fn volume(&self) -> Float;
    fn contains(&self, point: Point<Self::Dimension>) -> bool;
}

pub struct Cell<D: Dimension> {
    pub delaunay_point: PointIndex,
    pub center: Point<D>,
    pub index: CellIndex,
    pub points: Vec<Point<D>>,
    pub connected_cells: Vec<CellIndex>,
    pub is_boundary: bool,
}

fn sign(p1: Point2d, p2: Point2d, p3: Point2d) -> Float {
    (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
}

impl<D: Dimension> Cell<D> {
    pub fn point_windows(&self) -> impl Iterator<Item = (&Point<D>, &Point<D>)> {
        periodic_windows(&self.points)
    }

    pub fn iter_neighbours_and_faces<'a>(
        &'a self,
        grid: &'a VoronoiGrid<D>,
    ) -> impl Iterator<Item = (usize, Float, Point<D>)> + 'a {
        self.connected_cells
            .iter()
            .zip(self.point_windows())
            .map(|(c, (p1, p2))| {
                let face_area = p1.distance(*p2);
                let center_this_cell = self.center;
                let center_other_cell = grid.cells[*c].center;
                let normal = (center_other_cell - center_this_cell).normalize();
                (*c, face_area, normal)
            })
            .filter(|_| !self.is_boundary) // For now: return an empty iterator if this is a boundary cell
    }
}

impl DimensionCell for Cell<TwoD> {
    type Dimension = TwoD;

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
}

impl<D> From<&DelaunayTriangulation<D>> for VoronoiGrid<D>
where
    D: Dimension,
    DelaunayTriangulation<D>: Delaunay<D>,
{
    fn from(t: &DelaunayTriangulation<D>) -> Self {
        let mut map: StableHashMap<PointIndex, CellIndex> = StableHashMap::default();
        let point_to_tetra_map = point_to_tetra_map(t);
        let mut cells = vec![];
        for (i, (point_index, _)) in t.points.iter().enumerate() {
            map.insert(point_index, i);
        }
        for (point_index, _) in t.points.iter() {
            let mut points = vec![];
            let mut connected_cells = vec![];
            let tetras = &point_to_tetra_map[&point_index];
            for tetra in tetras.iter() {
                points.push(
                    t.get_tetra_data(&t.tetras[*tetra])
                        .get_center_of_circumcircle(),
                );
            }
            let mut is_boundary = false;
            for (t1, t2) in periodic_windows(tetras) {
                let common_face = t.tetras[*t1].get_common_face_with(&t.tetras[*t2]);
                if let Some(common_face) = common_face {
                    for other_point in t.faces[common_face].other_points(point_index) {
                        connected_cells.push(map[&other_point]);
                    }
                } else {
                    is_boundary = true;
                }
            }
            cells.push(Cell {
                center: t.points[point_index],
                index: map[&point_index],
                delaunay_point: point_index,
                points,
                connected_cells,
                is_boundary,
            });
        }
        VoronoiGrid { cells }
    }
}

fn point_to_tetra_map<D: Dimension>(
    triangulation: &DelaunayTriangulation<D>,
) -> StableHashMap<PointIndex, Vec<TetraIndex>>
where
    D: Dimension,
    DelaunayTriangulation<D>: Delaunay<D>,
{
    let mut map: StableHashMap<_, _> = triangulation
        .points
        .iter()
        .map(|(i, _)| (i, vec![]))
        .collect();
    for (tetra_index, tetra) in triangulation.tetras.iter() {
        for p in tetra.points() {
            map.get_mut(&p).unwrap().push(tetra_index);
        }
    }
    map
}
