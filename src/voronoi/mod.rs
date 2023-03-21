pub mod constructor;
pub mod delaunay;
mod indexed_arena;
pub mod math;
mod precision_error;
mod visualizer;

mod primitives;

mod utils;

mod cell;

use std::collections::HashSet;

use bevy::prelude::Resource;
use bevy::utils::StableHashMap;
pub use cell::Cell;
pub use cell::DimensionCell;
pub use delaunay::dimension::Dimension;
pub use delaunay::dimension::DimensionTetra;
pub use delaunay::DelaunayTriangulation;

use self::cell::CellConnection;
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

impl From<&DelaunayTriangulation<TwoD>> for VoronoiGrid<TwoD> {
    fn from(t: &DelaunayTriangulation<TwoD>) -> Self {
        let mut map: StableHashMap<PointIndex, CellIndex> = StableHashMap::default();
        let point_to_tetra_map = point_to_tetra_map(t);
        let mut cells = vec![];
        for (i, point_index) in t.iter_inner_points().enumerate() {
            map.insert(point_index, i);
        }
        for point_index in t.iter_inner_points() {
            let mut points = vec![];
            let mut connected_cells = vec![];
            let tetras = &point_to_tetra_map[&point_index];
            let mut prev_tetra: Option<TetraIndex> = None;
            let mut tetra = tetras[0];
            loop {
                let tetra_data = &t.tetras[tetra];
                let face = tetra_data
                    .faces()
                    .find(|face| {
                        if let Some(opp) = face.opposing {
                            let other_tetra_is_incident_with_cell = tetras.contains(&opp.tetra);
                            let other_tetra_is_prev_tetra = prev_tetra
                                .map(|prev_tetra| prev_tetra == opp.tetra)
                                .unwrap_or(false);
                            other_tetra_is_incident_with_cell && !other_tetra_is_prev_tetra
                        } else {
                            false
                        }
                    })
                    .unwrap();
                points.push(t.get_tetra_data(tetra_data).get_center_of_circumcircle());
                for other_point in t.faces[face.face].other_points(point_index) {
                    connected_cells.push(
                        map.get(&other_point)
                            .map(|i| CellConnection::ToInner(*i))
                            .unwrap_or(CellConnection::ToOuter),
                    );
                }
                prev_tetra = Some(tetra);
                tetra = face.opposing.unwrap().tetra;
                if tetra == tetras[0] {
                    break;
                }
            }

            cells.push(Cell {
                center: t.points[point_index],
                index: map[&point_index],
                delaunay_point: point_index,
                points,
                connected_cells,
            });
        }
        VoronoiGrid { cells }
    }
}

impl From<&DelaunayTriangulation<ThreeD>> for VoronoiGrid<ThreeD> {
    fn from(t: &DelaunayTriangulation<ThreeD>) -> Self {
        let mut map: StableHashMap<PointIndex, CellIndex> = StableHashMap::default();
        let point_to_tetra_map = point_to_tetra_map(t);
        let mut cells = vec![];
        for (i, point_index) in t.iter_inner_points().enumerate() {
            map.insert(point_index, i);
        }
        for point_index in t.iter_inner_points() {
            let mut points = vec![];
            let mut connected_cells = HashSet::new();
            let tetras = &point_to_tetra_map[&point_index];
            for tetra in tetras.iter() {
                let tetra_data = &t.tetras[*tetra];
                points.push(t.get_tetra_data(&tetra_data).get_center_of_circumcircle());
                let face = tetra_data.find_face_opposite(point_index);
                for other_point in t.faces[face.face].other_points(point_index) {
                    connected_cells.insert(
                        map.get(&other_point)
                            .map(|i| CellConnection::ToInner(*i))
                            .unwrap_or(CellConnection::ToOuter),
                    );
                }
            }

            cells.push(Cell {
                center: t.points[point_index],
                index: map[&point_index],
                delaunay_point: point_index,
                points,
                connected_cells: connected_cells.into_iter().collect(),
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
    use crate::vis;
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
        DelaunayTriangulation<D>: super::visualizer::Visualizable,
        Cell<D>: DimensionCell<Dimension = D>,
        VoronoiGrid<D>: for<'a> From<&'a DelaunayTriangulation<D>>,
    {
        perform_check_on_each_level_of_construction(|triangulation, num_inserted_points| {
            let grid: VoronoiGrid<D> = triangulation.into();
            let mut temp_vis = crate::voronoi::visualizer::Visualizer::default();
            for c in grid.cells.iter() {
                temp_vis.add(c);
            }
            temp_vis.add(triangulation);
            for lookup_point in D::get_lookup_points() {
                let containing_cell = get_containing_voronoi_cell(&grid, lookup_point);
                if num_inserted_points == 0 {
                    continue;
                }
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

#[cfg(test)]
mod quantitative_tests {
    use super::primitives::Point2d;
    use super::DelaunayTriangulation;
    use super::TwoD;
    use super::VoronoiGrid;
    use crate::test_utils::assert_float_is_close;
    use crate::voronoi::cell::CellConnection;
    use crate::voronoi::primitives::Point3d;
    use crate::voronoi::DimensionCell;
    use crate::voronoi::ThreeD;

    #[test]
    fn right_volume_and_face_areas_two_d() {
        let points = vec![
            Point2d::new(0.0, 0.0),
            Point2d::new(0.1, 0.9),
            Point2d::new(0.9, 0.2),
            Point2d::new(0.25, 0.25),
        ];
        let (t, map) = DelaunayTriangulation::<TwoD>::construct_from_iter(points.into_iter());
        let last_point_index = map.last().unwrap();
        let grid = VoronoiGrid::<TwoD>::from(&t);
        assert_eq!(grid.cells.len(), 4);
        // Find the cell associated with the (0.25, 0.25) point above. This cell should be a triangle.
        // The exact values of faces and normals are taken from constructing the grid by hand and inspecting ;)
        let cell = grid
            .cells
            .iter()
            .find(|cell| cell.delaunay_point == *last_point_index)
            .unwrap();
        assert_float_is_close(cell.volume(), 0.3968809165232358);
        for (neighbour_index, face_area, normal) in cell.iter_neighbours_and_faces(&grid) {
            if neighbour_index == CellConnection::ToInner(0) {
                assert_float_is_close(face_area, 1.0846512947129363);
                assert_float_is_close(normal.x, -0.5f64.sqrt());
                assert_float_is_close(normal.y, -0.5f64.sqrt());
            } else if neighbour_index == CellConnection::ToInner(1) {
                assert_float_is_close(face_area, 0.862988661979256);
                assert_float_is_close(normal.x, -0.22485950669875832);
                assert_float_is_close(normal.y, 0.9743911956946198);
            } else if neighbour_index == CellConnection::ToInner(2) {
                assert_float_is_close(face_area, 0.9638545380497548);
                assert_float_is_close(normal.x, 0.9970544855015816);
                assert_float_is_close(normal.y, -0.07669649888473688);
            } else {
                panic!()
            }
        }
    }

    #[test]
    fn right_volume_and_face_areas_three_d() {
        let points = vec![
            Point3d::new(0.0, 0.0, 0.0),
            Point3d::new(0.9, 0.1, 0.05),
            Point3d::new(0.05, 0.9, 0.1),
            Point3d::new(0.1, 0.05, 0.9),
            Point3d::new(0.25, 0.25, 0.25),
        ];
        let (t, map) = DelaunayTriangulation::<ThreeD>::construct_from_iter(points.into_iter());
        let last_point_index = map.last().unwrap();
        let grid = VoronoiGrid::<ThreeD>::from(&t);
        assert_eq!(grid.cells.len(), 5);
        // Find the cell associated with the (0.25, 0.25, 0.25) point above.
        // The exact values of faces and normals are taken from constructing the grid by hand and inspecting ;)
        let cell = grid
            .cells
            .iter()
            .find(|cell| cell.delaunay_point == *last_point_index)
            .unwrap();
        assert_eq!(cell.connected_cells.len(), 4);
        // assert_float_is_close(cell.volume(), 0.3968809165232358);
        // for (neighbour_index, face_area, normal) in cell.iter_neighbours_and_faces(&grid) {
        //     if neighbour_index == CellConnection::ToInner(0) {
        //         assert_float_is_close(face_area, 1.0846512947129363);
        //         assert_float_is_close(normal.x, -0.5f64.sqrt());
        //         assert_float_is_close(normal.y, -0.5f64.sqrt());
        //     } else if neighbour_index == CellConnection::ToInner(1) {
        //         assert_float_is_close(face_area, 0.862988661979256);
        //         assert_float_is_close(normal.x, -0.22485950669875832);
        //         assert_float_is_close(normal.y, 0.9743911956946198);
        //     } else if neighbour_index == CellConnection::ToInner(2) {
        //         assert_float_is_close(face_area, 0.9638545380497548);
        //         assert_float_is_close(normal.x, 0.9970544855015816);
        //         assert_float_is_close(normal.y, -0.07669649888473688);
        //     } else {
        //         panic!()
        //     }
        // }
    }
}
