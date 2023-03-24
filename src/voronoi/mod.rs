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
pub use cell::CellConnection;
pub use cell::DimensionCell;
pub use constructor::Constructor;
pub use delaunay::dimension::Dimension;
pub use delaunay::dimension::DimensionTetra;
use delaunay::Delaunay;
pub use delaunay::DelaunayTriangulation;
use delaunay::PointIndex;
pub use primitives::Point2d;
pub use primitives::Point3d;

pub type CellIndex = usize;

#[derive(Clone)]
pub struct TwoD;
#[derive(Clone)]
pub struct ThreeD;

type Point<D> = <D as Dimension>::Point;

#[derive(Resource)]
pub struct VoronoiGrid<D: Dimension> {
    pub cells: Vec<Cell<D>>,
}

impl<D: Dimension> From<&Constructor<D>> for VoronoiGrid<D>
where
    DelaunayTriangulation<D>: Delaunay<D>,
    Cell<D>: DimensionCell<Dimension = D>,
{
    fn from(c: &Constructor<D>) -> Self {
        c.construct()
    }
}

#[cfg(test)]
#[generic_tests::define]
mod tests {
    use bimap::BiMap;
    use ordered_float::OrderedFloat;

    use super::delaunay::dimension::Dimension;
    use super::delaunay::tests::perform_triangulation_check_on_each_level_of_construction;
    use super::delaunay::tests::TestableDimension;
    use super::delaunay::Delaunay;
    use super::delaunay::DelaunayTriangulation;
    use super::primitives::Point2d;
    use super::primitives::Point3d;
    use super::Cell;
    use super::Constructor;
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
            (0..30)
                .flat_map(move |i| (0..30).map(move |j| (i, j)))
                .map(|(i, j)| Point2d::new(0.01 * i as f64, 0.01 * j as f64))
                .collect()
        }
    }

    impl VoronoiTestDimension for ThreeD {
        fn get_lookup_points() -> Vec<Point3d> {
            (0..10)
                .flat_map(move |i| (0..10).map(move |j| (i, j)))
                .flat_map(move |(i, j)| (0..10).map(move |k| (i, j, k)))
                .map(|(i, j, k)| Point3d::new(0.1 * i as f64, 0.1 * j as f64, 0.1 * k as f64))
                .collect()
        }
    }

    pub fn perform_check_on_each_level_of_construction<D>(
        check: impl Fn(&Constructor<D>, usize) -> (),
    ) where
        D: Dimension + TestableDimension + Clone,
        DelaunayTriangulation<D>: Delaunay<D> + Clone,
        Cell<D>: DimensionCell<Dimension = D>,
    {
        perform_triangulation_check_on_each_level_of_construction(|t, num| {
            let map: BiMap<_, _> = t.points.iter().map(|(i, _)| i).enumerate().collect();
            check(&Constructor::new(t.clone(), map), num);
        });
    }

    #[test]
    fn voronoi_property<D: VoronoiTestDimension + TestableDimension>()
    where
        DelaunayTriangulation<D>: Delaunay<D>,
        DelaunayTriangulation<D>: super::visualizer::Visualizable,
        Cell<D>: DimensionCell<Dimension = D>,
        VoronoiGrid<D>: for<'a> From<&'a Constructor<D>>,
        <D as Dimension>::Point: std::fmt::Debug,
        D: Clone,
    {
        perform_check_on_each_level_of_construction(|constructor, num_inserted_points| {
            if num_inserted_points == 0 {
                return;
            }
            let mut num_found = 0;
            let grid: VoronoiGrid<D> = constructor.into();
            for lookup_point in D::get_lookup_points() {
                let containing_cell = get_containing_voronoi_cell(&grid, lookup_point);
                let closest_cell = grid
                    .cells
                    .iter()
                    .min_by_key(|cell| {
                        let p: D::Point = constructor.triangulation.points[cell.delaunay_point];
                        OrderedFloat(p.distance_squared(lookup_point))
                    })
                    .unwrap();
                if let Some(containing_cell) = containing_cell {
                    num_found += 1;
                    assert_eq!(containing_cell.delaunay_point, closest_cell.delaunay_point);
                }
            }
            assert!(num_found != 0); // Most likely this means that cell.contains doesn't work
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

#[cfg(test)]
mod quantitative_tests {
    use super::primitives::Point2d;
    use super::DelaunayTriangulation;
    use super::TwoD;
    use super::VoronoiGrid;
    use crate::test_utils::assert_float_is_close;
    use crate::voronoi::cell::CellConnection;
    use crate::voronoi::primitives::Point3d;
    use crate::voronoi::Constructor;
    use crate::voronoi::DimensionCell;
    use crate::voronoi::ThreeD;

    #[test]
    fn right_volume_and_face_areas_two_d() {
        let points = vec![
            (0, Point2d::new(0.0, 0.0)),
            (1, Point2d::new(0.1, 0.9)),
            (2, Point2d::new(0.9, 0.2)),
            (3, Point2d::new(0.25, 0.25)),
        ];
        let (t, map) = DelaunayTriangulation::<TwoD>::construct_from_iter(points.into_iter());
        let last_point_index = *map.get_by_left(&3).unwrap();
        let grid = VoronoiGrid::<TwoD>::from(&Constructor::new(t, map));
        assert_eq!(grid.cells.len(), 4);
        // Find the cell associated with the (0.25, 0.25) point above. This cell should be a triangle.
        // The exact values of faces and normals are taken from constructing the grid by hand and inspecting ;)
        let cell = grid
            .cells
            .iter()
            .find(|cell| cell.delaunay_point == last_point_index)
            .unwrap();
        assert_float_is_close(cell.volume(), 0.3968809165232358);
        for face in cell.faces.iter() {
            if face.connection == CellConnection::ToInner(0) {
                assert_float_is_close(face.area, 1.0846512947129363);
                assert_float_is_close(face.normal.x, -0.5f64.sqrt());
                assert_float_is_close(face.normal.y, -0.5f64.sqrt());
            } else if face.connection == CellConnection::ToInner(1) {
                assert_float_is_close(face.area, 0.862988661979256);
                assert_float_is_close(face.normal.x, -0.22485950669875832);
                assert_float_is_close(face.normal.y, 0.9743911956946198);
            } else if face.connection == CellConnection::ToInner(2) {
                assert_float_is_close(face.area, 0.9638545380497548);
                assert_float_is_close(face.normal.x, 0.9970544855015816);
                assert_float_is_close(face.normal.y, -0.07669649888473688);
            } else {
                panic!()
            }
        }
    }

    #[test]
    fn right_volume_and_face_areas_three_d() {
        let points = vec![
            (0, Point3d::new(0.0, 0.0, 0.0)),
            (1, Point3d::new(0.6, 0.1, 0.1)),
            (2, Point3d::new(0.1, 0.5, 0.1)),
            (3, Point3d::new(0.1, 0.1, 0.4)),
            (4, Point3d::new(0.1, 0.1, 0.1)),
        ];
        let (t, map) = DelaunayTriangulation::<ThreeD>::construct_from_iter(points.into_iter());
        let constructor = Constructor::new(t, map);
        let last_point_index = constructor.point_to_cell_map.get_by_left(&4).unwrap();
        let grid = VoronoiGrid::<ThreeD>::from(&constructor);
        assert_eq!(grid.cells.len(), 5);
        // Find the cell associated with the (0.25, 0.25, 0.25) point above.
        // The exact values of faces and normals are taken from constructing the grid by hand and inspecting ;)
        let cell = grid
            .cells
            .iter()
            .find(|cell| cell.delaunay_point == *last_point_index)
            .unwrap();
        assert_eq!(cell.faces.len(), 4);
        assert_eq!(cell.points.len(), 4);
        assert_float_is_close(cell.volume(), 0.0703125);
        for face in cell.faces.iter() {
            if face.connection == CellConnection::ToInner(0) {
                assert_float_is_close(face.area, 0.4871392896287468);
                assert_float_is_close(face.normal.x, -(1.0f64 / 3.0).sqrt());
                assert_float_is_close(face.normal.y, -(1.0f64 / 3.0).sqrt());
                assert_float_is_close(face.normal.z, -(1.0f64 / 3.0).sqrt());
            } else if face.connection == CellConnection::ToInner(1) {
                assert_float_is_close(face.area, 0.28125);
                assert_float_is_close(face.normal.x, 1.0);
                assert_float_is_close(face.normal.y, 0.0);
                assert_float_is_close(face.normal.z, 0.0);
            } else if face.connection == CellConnection::ToInner(2) {
                assert_float_is_close(face.area, 0.28125);
                assert_float_is_close(face.normal.x, 0.0);
                assert_float_is_close(face.normal.y, 1.0);
                assert_float_is_close(face.normal.z, 0.0);
            } else if face.connection == CellConnection::ToInner(3) {
                assert_float_is_close(face.area, 0.28125);
                assert_float_is_close(face.normal.x, 0.0);
                assert_float_is_close(face.normal.y, 0.0);
                assert_float_is_close(face.normal.z, 1.0);
            } else {
                panic!()
            }
        }
    }
}
