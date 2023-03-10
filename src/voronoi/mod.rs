pub mod constructor;
pub mod delaunay;
mod indexed_arena;
pub mod math;
mod precision_error;
mod visualizer;

mod primitives;

mod utils;

use std::f64::consts::PI;

use bevy::prelude::Resource;
use bevy::utils::StableHashMap;
pub use delaunay::DelaunayTriangulation;
use ordered_float::OrderedFloat;

use self::delaunay::dimension::Dimension;
use self::delaunay::dimension::DimensionFace;
use self::delaunay::dimension::DimensionTetra;
use self::delaunay::dimension::DimensionTetraData;
use self::delaunay::Delaunay;
use self::delaunay::PointIndex;
use self::delaunay::TetraIndex;
use self::primitives::Vector;
use self::utils::periodic_windows;
use crate::prelude::Float;

pub type CellIndex = usize;

pub struct TwoD;
pub struct ThreeD;
#[cfg(feature = "2d")]
pub type ActiveDimension = TwoD;
#[cfg(feature = "3d")]
pub type ActiveDimension = ThreeD;

type Point<D> = <D as Dimension>::Point;

#[derive(Resource)]
pub struct VoronoiGrid<D: Dimension> {
    pub cells: Vec<Cell<D>>,
}

pub struct Cell<D: Dimension> {
    pub delaunay_point: PointIndex,
    pub center: Point<D>,
    pub index: CellIndex,
    pub points: Vec<Point<D>>,
    pub connected_cells: Vec<CellIndex>,
    pub is_boundary: bool,
}

impl<D: Dimension> Cell<D> {
    pub fn point_windows(&self) -> impl Iterator<Item = (&Point<D>, &Point<D>)> {
        periodic_windows(&self.points)
    }

    pub fn contains(&self, _point: Point<D>) -> bool {
        todo!()
        // The following works in 2d but makes possibly no sense in 3d
        // let has_negative = self
        //     .point_windows()
        //     .map(|(p1, p2)| sign(point, *p1, *p2))
        //     .any(|s| s < 0.0);
        // let has_positive = self
        //     .point_windows()
        //     .map(|(p1, p2)| sign(point, *p1, *p2))
        //     .any(|s| s > 0.0);

        // !(has_negative && has_positive)
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

    #[cfg(feature = "2d")]
    pub fn size(&self) -> Float {
        (self.volume() / PI).sqrt()
    }

    #[cfg(feature = "2d")]
    pub fn volume(&self) -> Float {
        0.5 * self
            .point_windows()
            .map(|(p1, p2)| p1.x * p2.y - p2.x * p1.y)
            .sum::<Float>()
            .abs()
    }

    #[cfg(feature = "3d")]
    pub fn size(&self) -> Float {
        (3.0 * self.volume() / (4.0 * PI)).cbrt()
    }

    #[cfg(feature = "3d")]
    pub fn volume(&self) -> Float {
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
                    todo!()
                    // let other_point = t.faces[common_face].get_other_point(point_index);
                    // connected_cells.push(map[&other_point]);
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
    for (point_index, tetras) in map.iter_mut() {
        let point = triangulation.points[*point_index];
        tetras.sort_by_key(|t| {
            let p = triangulation
                .get_tetra_data(&triangulation.tetras[*t])
                .get_center_of_circumcircle();
            todo!()
            // let vec = p - point;
            // OrderedFloat(vec.x.atan2(vec.y))
        });
    }
    map
}

#[cfg(test)]
#[generic_tests::define]
mod tests {
    use ordered_float::OrderedFloat;

    use super::delaunay::dimension::Dimension;
    use super::delaunay::tests::perform_check_on_each_level_of_construction;
    use super::delaunay::tests::TestableDimension;
    use super::delaunay::Delaunay;
    use super::delaunay::DelaunayTriangulation;
    use super::primitives::Point2d;
    use super::primitives::Point3d;
    use super::Cell;
    use super::ThreeD;
    use super::TwoD;
    use super::VoronoiGrid;
    use crate::voronoi::primitives::point::Vector;

    #[instantiate_tests(<TwoD>)]
    mod two_d {}

    #[instantiate_tests(<ThreeD>)]
    mod three_d {}

    trait VoronoiTestDimension: Dimension {
        fn get_lookup_points() -> Vec<Self::Point>;
    }

    impl VoronoiTestDimension for TwoD {
        fn get_lookup_points() -> Vec<Point2d> {
            ((0..10).zip(0..10))
                .map(|(i, j)| Point2d::new(0.1 * i as f64, 0.1 * j as f64))
                .collect()
        }
    }

    impl VoronoiTestDimension for ThreeD {
        fn get_lookup_points() -> Vec<Point3d> {
            ((0..10).zip(0..10).zip(0..10))
                .map(|((i, j), k)| Point3d::new(0.1 * i as f64, 0.1 * j as f64, 0.1 * k as f64))
                .collect()
        }
    }

    #[test]
    fn voronoi_property<D: VoronoiTestDimension + TestableDimension>()
    where
        DelaunayTriangulation<D>: Delaunay<D>,
    {
        perform_check_on_each_level_of_construction(|triangulation, _| {
            let grid: VoronoiGrid<D> = triangulation.into();
            for lookup_point in D::get_lookup_points() {
                let containing_cell = get_containing_voronoi_cell(&grid, lookup_point);
                let closest_cell = grid
                    .cells
                    .iter()
                    .min_by_key(|cell| {
                        let p: D::Point = triangulation.points[cell.delaunay_point];
                        OrderedFloat(p.distance_squared(lookup_point))
                    })
                    .unwrap();
                if let Some(containing_cell) = containing_cell {
                    assert_eq!(containing_cell.delaunay_point, closest_cell.delaunay_point);
                }
            }
        });
    }

    fn get_containing_voronoi_cell<D: VoronoiTestDimension>(
        grid: &VoronoiGrid<D>,
        point: D::Point,
    ) -> Option<&Cell<D>> {
        grid.cells.iter().find(|cell| cell.contains(point))
    }
}
