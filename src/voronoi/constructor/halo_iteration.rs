use bevy::utils::StableHashSet;
use bimap::BiMap;

use super::super::delaunay::dimension::DTetra;
use super::super::delaunay::dimension::DTetraData;
use super::super::primitives::DVector;
use super::Cell;
use super::Delaunay;
use super::DimensionCell;
use super::Point;
use super::TetraIndex;
use crate::communication::DataByRank;
use crate::prelude::ParticleId;
use crate::voronoi::delaunay::PointIndex;
use crate::voronoi::delaunay::PointKind;
use crate::voronoi::primitives::Float;
use crate::voronoi::utils::Extent;
use crate::voronoi::Dimension;
use crate::voronoi::Triangulation;

/// Determines by how much the search radius is increased beyond the
/// mathematically necessary radius (equal to the radius of the
/// circumcircle/sphere of the tetra), in order to prevent numerical
/// problems due to floating point arithmetic.
const SEARCH_SAFETY_FACTOR: f64 = 1.05;

#[derive(Clone, Debug)]
pub struct SearchData<D: Dimension> {
    pub point: Point<D>,
    pub radius: Float,
    pub tetra_index: TetraIndex,
}

pub struct SearchResult<D: Dimension> {
    pub point: Point<D>,
    /// The index in the Vec of the corresponding RadiusSearchData
    /// that produced this result.
    pub search_index: TetraIndex,
    pub id: ParticleId,
}

impl<D: Dimension> SearchResult<D> {
    pub fn from_search(data: &SearchData<D>, point: Point<D>, particle_id: ParticleId) -> Self {
        Self {
            search_index: data.tetra_index,
            point,
            id: particle_id,
        }
    }
}

pub trait RadiusSearch<D: Dimension> {
    fn unique_radius_search(
        &mut self,
        data: Vec<SearchData<D>>,
    ) -> DataByRank<Vec<SearchResult<D>>>;
    fn determine_global_extent(&self) -> Option<Extent<Point<D>>>;
    fn everyone_finished(&mut self, num_undecided_this_rank: usize) -> bool;
}

pub(super) struct HaloIteration<D: Dimension, F> {
    pub triangulation: Triangulation<D>,
    search: F,
    checked_tetras: StableHashSet<TetraIndex>,
    pub haloes: BiMap<ParticleId, PointIndex>,
}

impl<D, F: RadiusSearch<D>> HaloIteration<D, F>
where
    D: Dimension,
    Triangulation<D>: Delaunay<D>,
    F: RadiusSearch<D>,
    Cell<D>: DimensionCell<Dimension = D>,
{
    pub fn new(triangulation: Triangulation<D>, search: F) -> Self {
        Self {
            triangulation,
            search,
            checked_tetras: StableHashSet::default(),
            haloes: BiMap::default(),
        }
    }

    pub fn run(&mut self) {
        while !self
            .search
            .everyone_finished(self.iter_remaining_tetras().count())
        {
            self.iterate();
        }
    }

    fn iterate(&mut self) {
        let search_data = self.get_radius_search_data();
        let mut newly_imported = self.search.unique_radius_search(search_data);
        let checked: StableHashSet<TetraIndex> = self.iter_remaining_tetras().collect();
        let mut tetras_with_new_points_in_vicinity = StableHashSet::default();
        for (rank, results) in newly_imported.drain_all() {
            for SearchResult {
                point,
                search_index,
                id: particle_id,
            } in results.into_iter()
            {
                let point_index = self.triangulation.insert(point, PointKind::Halo(rank));
                self.haloes.insert(particle_id, point_index);
                tetras_with_new_points_in_vicinity.insert(search_index);
            }
        }
        self.checked_tetras
            .extend(checked.difference(&tetras_with_new_points_in_vicinity));
    }

    fn get_radius_search_data(&self) -> Vec<SearchData<D>> {
        self.iter_remaining_tetras()
            .map(|t| {
                let tetra = &self.triangulation.tetras[t];
                let tetra_data = self.triangulation.get_tetra_data(tetra);
                let center = tetra_data.get_center_of_circumcircle();
                let sample_point = self.triangulation.points[tetra.points().next().unwrap()];
                let radius_circumcircle = center.distance(sample_point);
                let radius = SEARCH_SAFETY_FACTOR * radius_circumcircle;
                SearchData::<D> {
                    radius,
                    point: center,
                    tetra_index: t,
                }
            })
            .collect()
    }

    fn tetra_should_be_checked(&self, t: TetraIndex) -> bool {
        let tetra = &self.triangulation.tetras[t];
        tetra
            .points()
            .any(|p| self.triangulation.point_kinds[&p] == PointKind::Inner)
            && tetra
                .points()
                .all(|p| self.triangulation.point_kinds[&p] != PointKind::Outer)
    }

    fn iter_remaining_tetras(&self) -> impl Iterator<Item = TetraIndex> + '_ {
        self.triangulation
            .tetras
            .iter()
            .map(|(t, _)| t)
            .filter(|t| !self.checked_tetras.contains(t) && self.tetra_should_be_checked(*t))
    }
}

#[cfg(test)]
#[generic_tests::define]
mod tests {
    use bevy::utils::StableHashSet;

    use super::RadiusSearch;
    use super::SearchData;
    use super::SearchResult;
    use crate::communication::DataByRank;
    use crate::prelude::ParticleId;
    use crate::test_utils::assert_float_is_close_high_error;
    use crate::voronoi::constructor::Constructor;
    use crate::voronoi::delaunay::Delaunay;
    use crate::voronoi::primitives::point::DVector;
    use crate::voronoi::test_utils::TestDimension;
    use crate::voronoi::utils::get_extent;
    use crate::voronoi::utils::Extent;
    use crate::voronoi::Cell;
    use crate::voronoi::Dimension;
    use crate::voronoi::DimensionCell;
    use crate::voronoi::Point;
    use crate::voronoi::ThreeD;
    use crate::voronoi::Triangulation;
    use crate::voronoi::TriangulationData;
    use crate::voronoi::TwoD;
    use crate::voronoi::VoronoiGrid;

    #[instantiate_tests(<TwoD>)]
    mod two_d {}

    #[instantiate_tests(<ThreeD>)]
    mod three_d {}

    pub struct TestRadiusSearch<D: Dimension> {
        points: Vec<(ParticleId, Point<D>)>,
        extent: Extent<Point<D>>,
        already_sent: StableHashSet<ParticleId>,
    }

    impl<D: Dimension> RadiusSearch<D> for TestRadiusSearch<D> {
        fn unique_radius_search(
            &mut self,
            data: Vec<SearchData<D>>,
        ) -> DataByRank<Vec<SearchResult<D>>> {
            let fake_rank = 1;
            let mut d = DataByRank::empty();
            let mut results = vec![];
            for data in data.iter() {
                results.extend(
                    self.points
                        .iter()
                        .filter(|(_, p)| data.point.distance(*p) < data.radius)
                        .filter(|(j, _)| self.already_sent.insert(*j))
                        .map(move |(id, p)| SearchResult {
                            point: *p,
                            search_index: data.tetra_index,
                            id: *id,
                        }),
                )
            }
            d.insert(fake_rank, results);
            d
        }

        fn determine_global_extent(&self) -> Option<Extent<Point<D>>> {
            Some(self.extent.clone())
        }

        fn everyone_finished(&mut self, num_undecided_this_rank: usize) -> bool {
            num_undecided_this_rank == 0
        }
    }

    fn get_cell_for_particle<D: Dimension, 'a>(
        grid: &'a VoronoiGrid<D>,
        cons: &'a TriangulationData<D>,
        particle: ParticleId,
    ) -> &'a Cell<D> {
        grid.cells
            .iter()
            .find(|cell| {
                cell.delaunay_point == *cons.point_to_cell_map.get_by_left(&particle).unwrap()
            })
            .unwrap()
    }

    #[test]
    pub fn voronoi_grid_with_halo_points_is_the_same_as_without<D>()
    where
        D: Dimension + TestDimension,
        Triangulation<D>: Delaunay<D>,
        Point<D>: DVector,
        Cell<D>: DimensionCell<Dimension = D>,
    {
        // Obtain two point sets - the second of them shifted by some offset away from the first
        let (points1, points2) = D::get_example_point_sets_with_ids();
        let points = D::get_combined_point_set();
        // First construct the triangulation normally
        let full_constructor = Constructor::new(points.iter().cloned());
        // Now construct the triangulation of the first set using imported
        // halo particles imported from the other set.
        let extent = get_extent(points.iter().map(|(_, p)| p).cloned()).unwrap();
        let sub_constructor = Constructor::construct_from_iter(
            points1.iter().cloned(),
            TestRadiusSearch {
                points: points2,
                extent,
                already_sent: StableHashSet::default(),
            },
        );
        let data1 = full_constructor.data;
        let data2 = sub_constructor.data;
        let voronoi1 = data1.construct_voronoi();
        let voronoi2 = data2.construct_voronoi();
        for (id, _) in points1.iter() {
            let c1 = get_cell_for_particle(&voronoi1, &data1, *id);
            let c2 = get_cell_for_particle(&voronoi2, &data2, *id);
            // Infinite cells (i.e. those neighbouring the boundary) might very well
            // differ in exact shape because of the different encompassing tetras,
            // but this doesn't matter since they cannot be used anyways.
            if c1.is_infinite {
                assert!(c2.is_infinite);
                continue;
            }
            assert_eq!(c1.faces.len(), c2.faces.len());
            assert_float_is_close_high_error(c1.volume(), c2.volume());
        }
    }
}
