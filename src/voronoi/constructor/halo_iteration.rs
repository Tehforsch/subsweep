use super::super::delaunay::dimension::DTetra;
use super::Cell;
use super::DCell;
use super::Delaunay;
use super::Point;
use super::TetraIndex;
use crate::communication::DataByRank;
use crate::communication::Rank;
use crate::dimension::ActiveWrapType;
use crate::dimension::Dimension;
use crate::dimension::WrapType;
use crate::extent::Extent;
use crate::hash_map::BiMap;
use crate::prelude::ParticleId;
use crate::sweep::grid::ParticleType;
use crate::sweep::grid::PeriodicNeighbour;
use crate::sweep::grid::RemoteNeighbour;
use crate::sweep::grid::RemotePeriodicNeighbour;
use crate::voronoi::delaunay::Circumcircle;
use crate::voronoi::delaunay::PointIndex;
use crate::voronoi::delaunay::PointKind;
use crate::voronoi::primitives::Float;
use crate::voronoi::visualizer::Visualizable;
use crate::voronoi::CellIndex;
use crate::voronoi::DDimension;
use crate::voronoi::Triangulation;

/// Determines by how much all search radii should be larger than the
/// radius of the circumcircle/sphere of the tetra, in order to prevent numerical
/// problems due to floating point arithmetic.
const SEARCH_SAFETY_FACTOR: f64 = 1.001;

/// Determines by how much the search radii are increased between iterations.
/// If the factor is too low, large tetras will take a long time
/// to find all their haloes. If the factor is too high, we risk importing way
/// too many haloes than are needed to construct the proper triangulation.
const SEARCH_RADIUS_INCREASE_FACTOR: f64 = 1.6;

/// By how much to decrease/increase the initial maximally allowed search radius below the
/// "cartesian" cell size of side_length / num_particles_per_dimension
const INITIAL_SEARCH_RADIUS_GUESS_FACTOR: f64 = 1.5;

const SEARCH_CHUNK_SIZE: usize = 50000;

pub fn get_characteristic_length<D: Dimension>(max_side_length: f64, num_particles: usize) -> f64 {
    let num_particles_per_dim = (num_particles as f64).powf(1.0 / D::NUM as f64);
    (max_side_length / num_particles_per_dim) * INITIAL_SEARCH_RADIUS_GUESS_FACTOR
}

#[derive(Clone, Debug)]
pub struct SearchData<D: Dimension> {
    pub point: Point<D>,
    pub radius: Float,
}

impl<D: Dimension + DDimension> SearchData<D>
where
    Triangulation<D>: Delaunay<D>,
{
    fn new(
        undecided: &UndecidedTetraInfo,
        circumcircle: &Circumcircle<D>,
        triangulation: &Triangulation<D>,
        characteristic_length: f64,
    ) -> Self {
        let max_necessary_radius = circumcircle.radius * SEARCH_SAFETY_FACTOR;
        let radius = match undecided.search_radius {
            Some(radius) => (radius * SEARCH_RADIUS_INCREASE_FACTOR).min(max_necessary_radius),
            None => max_necessary_radius.min(characteristic_length),
        };
        let point = if radius >= max_necessary_radius {
            circumcircle.center
        } else {
            // If the radius is smaller than the circumcircle, we are really only
            // looking to find any close-by points (remote/periodic) to add to the triangulation
            // in order to show us that the tetra is not really as big as we currently think it is.
            // However, if we search for points around the center of the circumcircle, we might import
            // very far away points. As a slightly hacky way that seems to work in practice, we look
            // for points around any of the inner points that are part of the tetra in this case.
            let tetra = &triangulation.tetras[undecided.tetra];
            let p_index = tetra
                .points()
                .find(|p| triangulation.point_kinds[p] == PointKind::Inner)
                .unwrap();
            triangulation.get_original_point(p_index)
        };
        Self { point, radius }
    }
}

#[derive(Debug)]
pub struct SearchResult<D: Dimension> {
    pub point: Point<D>,
    pub id: ParticleId,
    pub periodic_wrap_type: WrapType<D>,
}

pub type SearchResults<D> = Vec<SearchResult<D>>;

pub trait RadiusSearch<D: Dimension> {
    fn radius_search(&mut self, data: Vec<SearchData<D>>) -> DataByRank<SearchResults<D>>;
    fn determine_global_extent(&self) -> Option<Extent<Point<D>>>;
    fn everyone_finished(&mut self, num_undecided_this_rank: usize) -> bool;
    fn num_points(&mut self) -> usize;
    fn rank(&self) -> Rank;
}

struct UndecidedTetraInfo {
    tetra: TetraIndex,
    search_radius: Option<Float>,
}

impl UndecidedTetraInfo {
    fn search_radius_large_enough<D: DDimension>(&self, circumcircle: &Circumcircle<D>) -> bool {
        self.search_radius.unwrap() >= circumcircle.radius * SEARCH_SAFETY_FACTOR
    }

    fn circumcircle<D: DDimension>(&self, triangulation: &Triangulation<D>) -> Circumcircle<D>
    where
        Triangulation<D>: Delaunay<D>,
    {
        triangulation.get_original_tetra_circumcircle(self.tetra)
    }
}

pub(super) struct HaloIteration<D: DDimension, F> {
    pub triangulation: Triangulation<D>,
    search: F,
    pub haloes: BiMap<CellIndex, PointIndex>,
    undecided_tetras: Vec<UndecidedTetraInfo>,
    characteristic_length: Float,
}

impl<D, F: RadiusSearch<D>> HaloIteration<D, F>
where
    D: DDimension<WrapType = ActiveWrapType>,
    Triangulation<D>: Delaunay<D>,
    F: RadiusSearch<D>,
    Cell<D>: DCell<Dimension = D>,
    SearchData<D>: Visualizable,
    Extent<Point<D>>: Visualizable,
{
    pub fn new(triangulation: Triangulation<D>, search: F, characteristic_length: Float) -> Self {
        let mut h = Self {
            triangulation,
            search,
            haloes: BiMap::default(),
            undecided_tetras: vec![],
            characteristic_length,
        };
        h.set_all_tetras_undecided();
        h
    }

    pub fn run(&mut self) {
        while !self.search.everyone_finished(self.undecided_tetras.len()) {
            self.iterate();
        }
    }

    fn iterate(&mut self) {
        let search_data = self.get_radius_search_data();
        #[cfg(feature = "vis")]
        crate::vis![
            &self.triangulation,
            &search_data,
            &self.search.determine_global_extent().unwrap()
        ];
        let search_results = self.search.radius_search(search_data);
        for (rank, results) in search_results.into_iter() {
            for SearchResult {
                point,
                id,
                periodic_wrap_type,
            } in results
            {
                let ptype = if rank == self.search.rank() {
                    ParticleType::LocalPeriodic(PeriodicNeighbour {
                        id,
                        periodic_wrap_type,
                    })
                } else {
                    if periodic_wrap_type.is_periodic() {
                        ParticleType::RemotePeriodic(RemotePeriodicNeighbour {
                            id,
                            rank,
                            periodic_wrap_type,
                        })
                    } else {
                        ParticleType::Remote(RemoteNeighbour { id, rank })
                    }
                };
                assert!(self.haloes.get_by_left(&ptype).is_none());
                let (point_index, changed_tetras) =
                    self.triangulation.insert(point, PointKind::Halo(rank));
                for tetra in changed_tetras.iter() {
                    if self.tetra_should_be_checked(*tetra) {
                        self.undecided_tetras
                            .push(self.get_undecided_tetra_info_for_new_tetra(*tetra));
                    }
                }
                self.haloes.insert(ptype, point_index);
            }
        }
    }

    fn get_radius_search_data(&mut self) -> Vec<SearchData<D>> {
        let chunk_size = SEARCH_CHUNK_SIZE.min(self.undecided_tetras.len());
        let (search_data, new_undecided): (Vec<_>, Vec<_>) = self
            .undecided_tetras
            .drain(..chunk_size)
            .filter(|undecided| self.triangulation.tetras.contains(undecided.tetra))
            .map(|mut undecided| {
                let circumcircle = undecided.circumcircle(&self.triangulation);
                let search_data = SearchData::new(
                    &undecided,
                    &circumcircle,
                    &self.triangulation,
                    self.characteristic_length,
                );
                undecided.search_radius = Some(search_data.radius);
                let large_enough = undecided.search_radius_large_enough(&circumcircle);
                (search_data, (undecided, large_enough))
            })
            .unzip();
        // Every tetra that has a larger circumcircle than the corresponding search radius
        // will need to be checked again later.
        self.undecided_tetras.extend(
            new_undecided
                .into_iter()
                .filter(|(t, large_enough)| {
                    assert!(self.triangulation.tetras.contains(t.tetra));
                    !large_enough
                })
                .map(|(t, _)| t),
        );
        search_data
    }

    fn set_all_tetras_undecided(&mut self) {
        self.undecided_tetras = self
            .triangulation
            .tetras
            .iter()
            .filter(|(tetra, _)| self.tetra_should_be_checked(*tetra))
            .map(|(tetra, _)| self.get_undecided_tetra_info_for_new_tetra(tetra))
            .collect();
    }

    fn get_undecided_tetra_info_for_new_tetra(&self, tetra: TetraIndex) -> UndecidedTetraInfo {
        UndecidedTetraInfo {
            tetra,
            search_radius: None,
        }
    }

    fn tetra_should_be_checked(&self, tetra: TetraIndex) -> bool {
        self.triangulation
            .tetras
            .get(tetra)
            .map(|tetra| {
                tetra
                    .points()
                    .any(|p| self.triangulation.point_kinds[&p] == PointKind::Inner)
            })
            .unwrap_or(false)
    }
}

#[cfg(test)]
#[generic_tests::define]
mod tests {
    use std::fmt::Debug;

    use super::RadiusSearch;
    use super::SearchData;
    use super::SearchResults;
    use crate::communication::DataByRank;
    use crate::communication::Rank;
    use crate::dimension::ActiveWrapType;
    use crate::dimension::Dimension;
    use crate::dimension::Point;
    use crate::dimension::WrapType;
    use crate::extent::Extent;
    use crate::prelude::Float;
    use crate::prelude::ParticleId;
    use crate::voronoi::constructor::halo_cache::HaloCache;
    use crate::voronoi::constructor::Constructor;
    use crate::voronoi::delaunay::dimension::DTetra;
    use crate::voronoi::delaunay::Delaunay;
    use crate::voronoi::delaunay::PointKind;
    use crate::voronoi::math::traits::DVector;
    use crate::voronoi::test_utils::TestDimension;
    use crate::voronoi::visualizer::Visualizable;
    use crate::voronoi::Cell;
    use crate::voronoi::CellIndex;
    use crate::voronoi::DCell;
    use crate::voronoi::DDimension;
    use crate::voronoi::Triangulation;
    use crate::voronoi::TriangulationData;
    use crate::voronoi::VoronoiGrid;

    #[cfg(feature = "2d")]
    #[instantiate_tests(<crate::dimension::TwoD>)]
    mod two_d {}

    #[cfg(feature = "3d")]
    #[instantiate_tests(<crate::dimension::ThreeD>)]
    mod three_d {}

    pub const OTHER_RANK: i32 = 1;

    #[derive(Clone)]
    pub struct TestRadiusSearch<D: DDimension> {
        points: Vec<(ParticleId, Point<D>)>,
        extent: Extent<Point<D>>,
        cache: HaloCache<D>,
    }

    impl<D: DDimension + Debug> RadiusSearch<D> for TestRadiusSearch<D> {
        fn radius_search(&mut self, data: Vec<SearchData<D>>) -> DataByRank<SearchResults<D>> {
            let mut d = DataByRank::empty();
            let mut new_haloes = vec![];
            for search in data.iter() {
                let result = self.cache.get_new_haloes(
                    OTHER_RANK,
                    self.points
                        .iter()
                        .filter(|(_, p)| search.point.distance(*p) <= search.radius)
                        .map(|(j, p)| (*p, *j, WrapType::<D>::default())),
                );
                new_haloes.extend(result);
            }
            d.insert(OTHER_RANK, new_haloes);
            d
        }

        fn determine_global_extent(&self) -> Option<Extent<Point<D>>> {
            Some(self.extent.clone())
        }

        fn everyone_finished(&mut self, num_undecided_this_rank: usize) -> bool {
            num_undecided_this_rank == 0
        }

        fn rank(&self) -> Rank {
            0
        }

        fn num_points(&mut self) -> usize {
            self.points.len()
        }
    }

    fn get_cell_for_local_particle<D: DDimension, 'a>(
        grid: &'a VoronoiGrid<D>,
        cons: &'a TriangulationData<D>,
        particle: ParticleId,
    ) -> &'a Cell<D> {
        grid.cells
            .iter()
            .find(|cell| {
                cell.delaunay_point
                    == *cons
                        .point_to_cell_map
                        .get_by_left(&CellIndex::Local(particle))
                        .unwrap()
            })
            .unwrap()
    }

    fn all_points_in_radius_imported<D>(
        sub_triangulation_data: &TriangulationData<D>,
        points: Vec<(ParticleId, Point<D>)>,
    ) where
        D: DDimension + TestDimension + Clone + Debug,
        Triangulation<D>: Delaunay<D>,
        Cell<D>: DCell<Dimension = D>,
    {
        for (t, tetra) in sub_triangulation_data.triangulation.tetras.iter() {
            if tetra
                .points()
                .all(|p| sub_triangulation_data.triangulation.point_kinds[&p] != PointKind::Inner)
            {
                continue;
            }
            let c = sub_triangulation_data
                .triangulation
                .get_original_tetra_circumcircle(t);
            let search = SearchData::<D> {
                point: c.center,
                radius: c.radius,
            };
            let points_in_radius = points
                .iter()
                .filter(|(_, p)| p.distance(search.point) < search.radius);
            for (id, _) in points_in_radius {
                assert!(sub_triangulation_data
                    .triangulation
                    .iter_original_points()
                    .any(|(p_index, _)| {
                        sub_triangulation_data
                            .point_to_cell_map
                            .get_by_right(&p_index)
                            .map(|cell_index| {
                                if *cell_index == CellIndex::Boundary {
                                    false
                                } else {
                                    cell_index.unwrap_id() == *id
                                }
                            })
                            .unwrap_or(false)
                    }));
            }
        }
    }

    fn assert_float_is_close_high_error(x: Float, y: Float) {
        assert!(
            (x - y).abs() / (x + y).abs() < 1e-2,
            "{} {} (diff: {})",
            x,
            y,
            (x - y).abs() / (x + y),
        )
    }

    fn get_combined_point_set<D: TestDimension>() -> Vec<(ParticleId, D::Point)> {
        let (p1, p2) = get_example_point_sets_with_ids::<D>();
        p1.into_iter().chain(p2.into_iter()).collect()
    }

    fn get_example_point_sets_with_ids<D: TestDimension>(
    ) -> (Vec<(ParticleId, D::Point)>, Vec<(ParticleId, D::Point)>) {
        let p1 = D::get_example_point_set_num(100, 0);
        let p2 = D::get_surrounding_points(100);
        let len_p1 = p1.len();
        (
            p1.into_iter()
                .enumerate()
                .map(|(i, p)| (ParticleId::test(i), p))
                .collect(),
            p2.into_iter()
                .enumerate()
                .map(|(i, p)| {
                    (
                        ParticleId {
                            index: (len_p1 + i) as u32,
                            rank: OTHER_RANK,
                        },
                        p,
                    )
                })
                .collect(),
        )
    }

    #[test]
    pub fn voronoi_grid_with_halo_points_is_the_same_as_without<D>()
    where
        D: DDimension
            + TestDimension
            + Clone
            + Debug
            + Dimension<WrapType = ActiveWrapType>
            + Default,
        Triangulation<D>: Delaunay<D>,
        Cell<D>: DCell<Dimension = D> + Debug,
        SearchData<D>: Visualizable,
        Extent<Point<D>>: Visualizable,
    {
        // Obtain two point sets - the second of them shifted by some offset away from the first
        let (local_points, remote_points) = get_example_point_sets_with_ids::<D>();
        let points = get_combined_point_set::<D>();
        // First construct the triangulation normally
        let full_constructor = Constructor::new(points.iter().cloned());
        // Now construct the triangulation of the first set using imported
        // halo particles imported from the other set.
        let extent = Extent::from_points(points.iter().map(|(_, p)| p).cloned()).unwrap();
        let sub_constructor = Constructor::construct_from_iter(
            local_points.iter().cloned(),
            TestRadiusSearch {
                points: remote_points,
                extent,
                cache: HaloCache::default(),
            },
            None,
        );
        let full_data = full_constructor.data;
        let sub_data = sub_constructor.data;
        let full_voronoi = full_data.construct_voronoi();
        let sub_voronoi = sub_data.construct_voronoi();
        assert!(
            sub_voronoi.cells.len() != full_voronoi.cells.len(),
            "All haloes imported - this test is not super useful in this way."
        );
        all_points_in_radius_imported(&sub_data, points);
        for (id, _) in local_points.iter() {
            let c1 = get_cell_for_local_particle(&full_voronoi, &full_data, *id);
            let c2 = get_cell_for_local_particle(&sub_voronoi, &sub_data, *id);
            assert_eq!(c1.is_infinite, c2.is_infinite);
            // Infinite cells (i.e. those neighbouring the boundary) might very well
            // differ in exact shape because of the different encompassing tetras,
            // but this doesn't matter since they cannot be used anyways.
            if c1.is_infinite {
                continue;
            }
            assert_eq!(c1.faces.len(), c2.faces.len());
            // Due to the fact that points can be re-arranged
            // (cyclically) inside of the polygonal faces of the
            // voronoi grid, the numerical error can be quite large
            // here. This only affects the face areas but not the
            // normals. However, the face areas are used to compute
            // the volumes of the cell, which leads to high errors.
            assert_float_is_close_high_error(c1.volume(), c2.volume());
        }
    }
}
