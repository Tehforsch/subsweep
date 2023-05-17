mod cell;
pub mod constructor;
pub mod delaunay;
mod indexed_arena;
pub mod math;
mod primitives;
#[cfg(test)]
mod test_utils;
pub mod triangulation_data;
mod utils;
pub mod visualizer;

use bevy::prelude::Resource;
pub use cell::Cell;
pub use cell::DCell;
pub use constructor::parallel::plugin::construct_grid_system;
pub use constructor::Constructor;
pub use delaunay::dimension::DDimension;
pub use delaunay::dimension::DTetra;
use delaunay::Delaunay;
use delaunay::PointIndex;
pub use delaunay::Triangulation;
pub use math::traits::DVector;
pub use math::traits::MinMax;
pub use primitives::Point2d;
pub use primitives::Point3d;
pub use triangulation_data::TriangulationData;

use crate::sweep::grid::ParticleType;

pub type CellIndex = ParticleType;

#[derive(Resource)]
pub struct VoronoiGrid<D: DDimension> {
    pub cells: Vec<Cell<D>>,
}

impl<D: DDimension> From<&TriangulationData<D>> for VoronoiGrid<D>
where
    Triangulation<D>: Delaunay<D>,
    Cell<D>: DCell<Dimension = D>,
{
    fn from(c: &TriangulationData<D>) -> Self {
        c.construct_voronoi()
    }
}

#[cfg(test)]
#[generic_tests::define]
mod tests {
    use bimap::BiMap;
    use ordered_float::OrderedFloat;

    use super::delaunay::dimension::DDimension;
    use super::delaunay::tests::perform_triangulation_check_on_each_level_of_construction;
    use super::delaunay::Delaunay;
    use super::delaunay::Triangulation;
    use super::test_utils::TestDimension;
    use super::Cell;
    use super::DCell;
    use super::TriangulationData;
    use super::VoronoiGrid;
    use crate::dimension::ThreeD;
    use crate::dimension::TwoD;
    use crate::prelude::ParticleId;
    use crate::sweep::grid::ParticleType;
    use crate::voronoi::DVector;

    #[instantiate_tests(<TwoD>)]
    mod two_d {}

    #[instantiate_tests(<ThreeD>)]
    mod three_d {}

    pub fn perform_check_on_each_level_of_construction<D>(
        check: impl Fn(&TriangulationData<D>, usize) -> (),
    ) where
        D: DDimension + TestDimension + Clone,
        Triangulation<D>: Delaunay<D> + Clone,
        Cell<D>: DCell<Dimension = D>,
    {
        perform_triangulation_check_on_each_level_of_construction(|t, num| {
            let map: BiMap<_, _> = t
                .points
                .iter()
                .enumerate()
                .map(|(i, (p, _))| (ParticleType::Local(ParticleId::test(i)), p))
                .collect();
            check(
                &TriangulationData::from_triangulation_and_map(t.clone(), map),
                num,
            );
        });
    }

    #[test]
    fn voronoi_property<D: TestDimension>()
    where
        Triangulation<D>: Delaunay<D>,
        Cell<D>: DCell<Dimension = D>,
        VoronoiGrid<D>: for<'a> From<&'a TriangulationData<D>>,
        D: Clone,
    {
        perform_check_on_each_level_of_construction(|data, num_inserted_points| {
            if num_inserted_points == 0 {
                return;
            }
            let mut num_found = 0;
            let grid: VoronoiGrid<D> = data.into();
            for lookup_point in D::get_lookup_points() {
                let containing_cell = get_containing_voronoi_cell(&grid, lookup_point);
                let closest_cell = grid
                    .cells
                    .iter()
                    .min_by_key(|cell| {
                        let p: D::Point = data.triangulation.points[cell.delaunay_point];
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
        D: TestDimension,
        Cell<D>: DCell<Dimension = D>,
    {
        grid.cells.iter().find(|cell| cell.contains(point))
    }
}

#[cfg(test)]
mod quantitative_tests {
    use super::VoronoiGrid;
    use crate::prelude::ParticleId;
    use crate::sweep::grid::ParticleType;
    use crate::test_utils::assert_float_is_close;
    use crate::voronoi::Constructor;
    use crate::voronoi::DCell;

    #[cfg(feature = "2d")]
    #[test]
    fn right_volume_and_face_areas_two_d() {
        use super::primitives::Point2d;
        use crate::dimension::TwoD;
        let points = vec![
            (ParticleId(0), Point2d::new(0.0, 0.0)),
            (ParticleId(1), Point2d::new(0.1, 0.9)),
            (ParticleId(2), Point2d::new(0.9, 0.2)),
            (ParticleId(3), Point2d::new(0.25, 0.25)),
        ];
        let cons = Constructor::new(points.into_iter());
        let last_point_index = cons
            .get_point_by_cell(ParticleType::Local(ParticleId(3)))
            .unwrap();
        let grid: VoronoiGrid<TwoD> = cons.voronoi();
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
            if face.connection == ParticleType::Local(ParticleId(0)) {
                assert_float_is_close(face.area, 1.0846512947129363);
                assert_float_is_close(face.normal.x, -0.5f64.sqrt());
                assert_float_is_close(face.normal.y, -0.5f64.sqrt());
            } else if face.connection == ParticleType::Local(ParticleId(1)) {
                assert_float_is_close(face.area, 0.862988661979256);
                assert_float_is_close(face.normal.x, -0.22485950669875832);
                assert_float_is_close(face.normal.y, 0.9743911956946198);
            } else if face.connection == ParticleType::Local(ParticleId(2)) {
                assert_float_is_close(face.area, 0.9638545380497548);
                assert_float_is_close(face.normal.x, 0.9970544855015816);
                assert_float_is_close(face.normal.y, -0.07669649888473688);
            } else {
                panic!()
            }
        }
    }

    #[cfg(feature = "3d")]
    #[test]
    fn right_volume_and_face_areas_three_d() {
        use crate::dimension::ThreeD;
        use crate::voronoi::primitives::Point3d;
        let points = vec![
            (ParticleId::test(0), Point3d::new(0.0, 0.0, 0.0)),
            (ParticleId::test(1), Point3d::new(0.6, 0.1, 0.1)),
            (ParticleId::test(2), Point3d::new(0.1, 0.5, 0.1)),
            (ParticleId::test(3), Point3d::new(0.1, 0.1, 0.4)),
            (ParticleId::test(4), Point3d::new(0.1, 0.1, 0.1)),
        ];
        let cons = Constructor::new(points.into_iter());
        let last_point_index = cons
            .get_point_by_cell(ParticleType::Local(ParticleId::test(4)))
            .unwrap();
        let grid: VoronoiGrid<ThreeD> = cons.voronoi();
        assert_eq!(grid.cells.len(), 5);
        // Find the cell associated with the (0.25, 0.25, 0.25) point above.
        // The exact values of faces and normals are taken from constructing the grid by hand and inspecting ;)
        let cell = grid
            .cells
            .iter()
            .find(|cell| cell.delaunay_point == last_point_index)
            .unwrap();
        assert_eq!(cell.faces.len(), 4);
        assert_eq!(cell.points.len(), 4);
        assert_float_is_close(cell.volume(), 0.0703125);
        for face in cell.faces.iter() {
            if face.connection == ParticleType::Local(ParticleId::test(0)) {
                assert_float_is_close(face.area, 0.4871392896287468);
                assert_float_is_close(face.normal.x, -(1.0f64 / 3.0).sqrt());
                assert_float_is_close(face.normal.y, -(1.0f64 / 3.0).sqrt());
                assert_float_is_close(face.normal.z, -(1.0f64 / 3.0).sqrt());
            } else if face.connection == ParticleType::Local(ParticleId::test(1)) {
                assert_float_is_close(face.area, 0.28125);
                assert_float_is_close(face.normal.x, 1.0);
                assert_float_is_close(face.normal.y, 0.0);
                assert_float_is_close(face.normal.z, 0.0);
            } else if face.connection == ParticleType::Local(ParticleId::test(2)) {
                assert_float_is_close(face.area, 0.28125);
                assert_float_is_close(face.normal.x, 0.0);
                assert_float_is_close(face.normal.y, 1.0);
                assert_float_is_close(face.normal.z, 0.0);
            } else if face.connection == ParticleType::Local(ParticleId::test(3)) {
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
