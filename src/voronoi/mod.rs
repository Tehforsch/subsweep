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
pub use cell::Cell;
pub use cell::DimensionCell;
pub use delaunay::DelaunayTriangulation;

use self::delaunay::dimension::Dimension;
use self::delaunay::PointIndex;

pub type CellIndex = usize;

pub struct TwoD;
pub struct ThreeD;

type Point<D> = <D as Dimension>::Point;

#[derive(Resource)]
pub struct VoronoiGrid<D: Dimension> {
    pub cells: Vec<Cell<D>>,
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
