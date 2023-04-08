use super::super::delaunay::dimension::DTetra;
use super::super::delaunay::dimension::DTetraData;
use super::super::primitives::DVector;
use super::Cell;
use super::DCell;
use super::Delaunay;
use super::Point;
use super::TetraIndex;
use crate::communication::DataByRank;
use crate::hash_map::BiMap;
use crate::hash_map::HashSet;
use crate::prelude::ParticleId;
use crate::voronoi::delaunay::PointIndex;
use crate::voronoi::delaunay::PointKind;
use crate::voronoi::primitives::Float;
use crate::voronoi::utils::Extent;
use crate::voronoi::DDimension;
use crate::voronoi::Triangulation;

/// Determines by how much the search radius is increased beyond the
/// mathematically necessary radius (equal to the radius of the
/// circumcircle/sphere of the tetra), in order to prevent numerical
/// problems due to floating point arithmetic.
const SEARCH_SAFETY_FACTOR: f64 = 1.05;

#[derive(Clone, Debug)]
pub struct SearchData<D: DDimension> {
    pub point: Point<D>,
    pub radius: Float,
    pub tetra_index: TetraIndex,
}

#[derive(Debug)]
pub struct SearchResult<D: DDimension> {
    pub point: Point<D>,
    pub id: ParticleId,
}

pub struct SearchResults<D: DDimension> {
    pub new_haloes: Vec<SearchResult<D>>,
    pub undecided_tetras: Vec<TetraIndex>,
}

pub trait RadiusSearch<D: DDimension> {
    fn radius_search(&mut self, data: Vec<SearchData<D>>) -> DataByRank<SearchResults<D>>;
    fn determine_global_extent(&self) -> Option<Extent<Point<D>>>;
    fn everyone_finished(&mut self, num_undecided_this_rank: usize) -> bool;
}

pub(super) struct HaloIteration<D: DDimension, F> {
    pub triangulation: Triangulation<D>,
    search: F,
    decided_tetras: HashSet<TetraIndex>,
    pub haloes: BiMap<ParticleId, PointIndex>,
}

impl<D, F: RadiusSearch<D>> HaloIteration<D, F>
where
    D: DDimension,
    Triangulation<D>: Delaunay<D>,
    F: RadiusSearch<D>,
    Cell<D>: DCell<Dimension = D>,
{
    pub fn new(triangulation: Triangulation<D>, search: F) -> Self {
        Self {
            triangulation,
            search,
            decided_tetras: HashSet::default(),
            haloes: BiMap::default(),
        }
    }

    pub fn run(&mut self) {
        while !self
            .search
            .everyone_finished(self.iter_undecided_tetras().count())
        {
            self.iterate();
        }
    }

    fn iterate(&mut self) {
        let search_data = self.get_radius_search_data();
        let mut newly_decided: HashSet<TetraIndex> =
            search_data.iter().map(|d| d.tetra_index).collect();
        let search_results = self.search.radius_search(search_data);
        for (rank, results) in search_results.into_iter() {
            for SearchResult {
                point,
                id: particle_id,
            } in results.new_haloes
            {
                assert!(self.haloes.get_by_left(&particle_id).is_none());
                let point_index = self.triangulation.insert(point, PointKind::Halo(rank));
                self.haloes.insert(particle_id, point_index);
            }
            for t in results.undecided_tetras.into_iter() {
                newly_decided.remove(&t);
            }
        }
        self.decided_tetras.extend(newly_decided);
    }

    fn get_radius_search_data(&self) -> Vec<SearchData<D>> {
        self.iter_undecided_tetras()
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
    }

    fn iter_undecided_tetras(&self) -> impl Iterator<Item = TetraIndex> + '_ {
        self.triangulation
            .tetras
            .iter()
            .map(|(t, _)| t)
            .filter(|t| !self.decided_tetras.contains(t) && self.tetra_should_be_checked(*t))
    }
}

#[cfg(test)]
#[generic_tests::define]
mod tests {
    use std::fmt::Debug;

    use super::HaloIteration;
    use super::RadiusSearch;
    use super::SearchData;
    use super::SearchResults;
    use crate::communication::DataByRank;
    use crate::dimension::ThreeD;
    use crate::dimension::TwoD;
    use crate::prelude::ParticleId;
    use crate::test_utils::assert_float_is_close_high_error;
    use crate::voronoi::constructor::halo_cache::CachedSearchResult;
    use crate::voronoi::constructor::halo_cache::HaloCache;
    use crate::voronoi::constructor::Constructor;
    use crate::voronoi::delaunay::Delaunay;
    use crate::voronoi::primitives::point::DVector;
    use crate::voronoi::test_utils::TestDimension;
    use crate::voronoi::utils::get_extent;
    use crate::voronoi::utils::Extent;
    use crate::voronoi::Cell;
    use crate::voronoi::DCell;
    use crate::voronoi::DDimension;
    use crate::voronoi::Point;
    use crate::voronoi::Triangulation;
    use crate::voronoi::TriangulationData;
    use crate::voronoi::VoronoiGrid;

    #[instantiate_tests(<TwoD>)]
    mod two_d {}

    #[instantiate_tests(<ThreeD>)]
    mod three_d {}

    #[derive(Clone)]
    pub struct TestRadiusSearch<D: DDimension> {
        points: Vec<(ParticleId, Point<D>)>,
        extent: Extent<Point<D>>,
        cache: HaloCache,
    }

    impl<D: DDimension + Debug> RadiusSearch<D> for TestRadiusSearch<D> {
        fn radius_search(&mut self, data: Vec<SearchData<D>>) -> DataByRank<SearchResults<D>> {
            let fake_rank = 1;
            let mut d = DataByRank::empty();
            let mut new_haloes = vec![];
            let mut undecided_tetras = vec![];
            for search in data.iter() {
                let result = self.cache.get_closest_new::<D>(
                    fake_rank,
                    search.point,
                    self.points
                        .iter()
                        .filter(|(_, p)| search.point.distance(*p) < search.radius)
                        .map(|(j, p)| (*p, *j)),
                );
                match result {
                    CachedSearchResult::NothingNew => {}
                    CachedSearchResult::NewPoint(result) => {
                        new_haloes.push(result);
                        undecided_tetras.push(search.tetra_index);
                    }
                    CachedSearchResult::NewPointThatHasJustBeenExported => {
                        undecided_tetras.push(search.tetra_index);
                    }
                }
            }
            self.cache.flush();
            d.insert(
                fake_rank,
                SearchResults {
                    new_haloes: new_haloes,
                    undecided_tetras,
                },
            );
            d
        }

        fn determine_global_extent(&self) -> Option<Extent<Point<D>>> {
            Some(self.extent.clone())
        }

        fn everyone_finished(&mut self, num_undecided_this_rank: usize) -> bool {
            num_undecided_this_rank == 0
        }
    }

    fn get_cell_for_particle<D: DDimension, 'a>(
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

    fn all_points_in_radius_imported<D>(
        sub_triangulation_data: &TriangulationData<D>,
        points: Vec<(ParticleId, Point<D>)>,
        extent: Extent<Point<D>>,
    ) where
        D: DDimension + TestDimension + Clone + Debug,
        Triangulation<D>: Delaunay<D>,
        Point<D>: DVector,
        Cell<D>: DCell<Dimension = D>,
    {
        let search = TestRadiusSearch {
            points: vec![],
            extent,
            cache: HaloCache::default(),
        };
        let halo_iteration =
            HaloIteration::new(sub_triangulation_data.triangulation.clone(), search.clone());
        for data in halo_iteration.get_radius_search_data() {
            let points_in_radius = points
                .iter()
                .filter(|(_, p)| p.distance(data.point) < data.radius);
            for (id, _) in points_in_radius {
                assert!(sub_triangulation_data
                    .triangulation
                    .points
                    .iter()
                    .any(|(p_index, _)| {
                        sub_triangulation_data
                            .point_to_cell_map
                            .get_by_right(&p_index)
                            == Some(id)
                    }));
            }
        }
    }

    #[test]
    pub fn voronoi_grid_with_halo_points_is_the_same_as_without<D>()
    where
        D: DDimension + TestDimension + Clone + Debug,
        Triangulation<D>: Delaunay<D>,
        Point<D>: DVector,
        Cell<D>: DCell<Dimension = D> + Debug,
    {
        // Obtain two point sets - the second of them shifted by some offset away from the first
        let (local_points, remote_points) = D::get_example_point_sets_with_ids();
        let points = D::get_combined_point_set();
        // First construct the triangulation normally
        let full_constructor = Constructor::new(points.iter().cloned());
        // Now construct the triangulation of the first set using imported
        // halo particles imported from the other set.
        let extent = get_extent(points.iter().map(|(_, p)| p).cloned()).unwrap();
        let sub_constructor = Constructor::construct_from_iter(
            local_points.iter().cloned(),
            TestRadiusSearch {
                points: remote_points.clone(),
                extent: extent.clone(),
                cache: HaloCache::default(),
            },
        );
        let full_data = full_constructor.data;
        let sub_data = sub_constructor.data;
        let full_voronoi = full_data.construct_voronoi();
        let sub_voronoi = sub_data.construct_voronoi();
        all_points_in_radius_imported(&sub_data, points.clone(), extent);
        for (id, _) in local_points.iter() {
            let c1 = get_cell_for_particle(&full_voronoi, &full_data, *id);
            let c2 = get_cell_for_particle(&sub_voronoi, &sub_data, *id);
            assert_eq!(c1.is_infinite, c2.is_infinite);
            // Infinite cells (i.e. those neighbouring the boundary) might very well
            // differ in exact shape because of the different encompassing tetras,
            // but this doesn't matter since they cannot be used anyways.
            if c1.is_infinite {
                continue;
            }
            assert_eq!(c1.faces.len(), c2.faces.len());
            assert_float_is_close_high_error(c1.volume(), c2.volume());
        }
    }
}
