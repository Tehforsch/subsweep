pub mod constructor;
pub mod delaunay;
mod indexed_arena;
pub mod math;
mod precision_error;
mod visualizer;

mod primitives;

mod utils;

mod cell;

use bevy::prelude::Resource;
use bevy::utils::StableHashMap;
pub use cell::Cell;
pub use cell::DimensionCell;
pub use delaunay::dimension::Dimension;
pub use delaunay::dimension::DimensionTetra;
pub use delaunay::DelaunayTriangulation;

use self::delaunay::dimension::DimensionTetraData;
use self::delaunay::Delaunay;
use self::delaunay::PointIndex;
use self::delaunay::TetraIndex;
use crate::voronoi::delaunay::dimension::DimensionFace;

pub type CellIndex = usize;

pub struct TwoD;
pub struct ThreeD;

type Point<D> = <D as Dimension>::Point;

#[derive(Resource)]
pub struct VoronoiGrid<D: Dimension> {
    pub cells: Vec<Cell<D>>,
}

impl From<&DelaunayTriangulation<ThreeD>> for VoronoiGrid<ThreeD> {
    fn from(t: &DelaunayTriangulation<ThreeD>) -> Self {
        todo!()
    }
}

impl From<&DelaunayTriangulation<TwoD>> for VoronoiGrid<TwoD> {
    fn from(t: &DelaunayTriangulation<TwoD>) -> Self {
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
            let mut prev_tetra: Option<TetraIndex> = None;
            let mut tetra = tetras[0];
            let is_boundary = loop {
                let tetra_data = &t.tetras[tetra];
                let face = tetra_data.faces().find(|face| {
                    if let Some(opp) = face.opposing {
                        let other_tetra_is_incident_with_cell = tetras.contains(&opp.tetra);
                        let other_tetra_is_prev_tetra = prev_tetra
                            .map(|prev_tetra| prev_tetra == opp.tetra)
                            .unwrap_or(false);
                        other_tetra_is_incident_with_cell && !other_tetra_is_prev_tetra
                    } else {
                        false
                    }
                });
                if let Some(face) = face {
                    points.push(t.get_tetra_data(tetra_data).get_center_of_circumcircle());
                    for other_point in t.faces[face.face].other_points(point_index) {
                        connected_cells.push(map[&other_point]);
                    }
                    prev_tetra = Some(tetra);
                    tetra = face.opposing.unwrap().tetra;
                    if tetra == tetras[0] {
                        break false;
                    }
                } else {
                    break true;
                }
            };

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
    use super::DimensionCell;
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
        Cell<D>: DimensionCell<Dimension = D>,
        VoronoiGrid<D>: for<'a> From<&'a DelaunayTriangulation<D>>,
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

    fn get_containing_voronoi_cell<D>(grid: &VoronoiGrid<D>, point: D::Point) -> Option<&Cell<D>>
    where
        D: VoronoiTestDimension,
        Cell<D>: DimensionCell<Dimension = D>,
    {
        grid.cells.iter().find(|cell| cell.contains(point))
    }
}
