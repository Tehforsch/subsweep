use std::hash::Hash;

use bevy::utils::StableHashSet;
use generational_arena::Index;
use mpi::traits::Equivalence;

use super::Point;
use super::TetraIndex;
use crate::voronoi::primitives::Float;
use crate::voronoi::Dimension;

#[derive(Equivalence, Clone, Copy)]
pub struct TetraIndexSend {
    gen: usize,
    index: u64,
}

impl From<TetraIndex> for TetraIndexSend {
    fn from(value: TetraIndex) -> Self {
        let (gen, index) = value.0.into_raw_parts();
        Self { gen, index }
    }
}

impl From<TetraIndexSend> for TetraIndex {
    fn from(value: TetraIndexSend) -> Self {
        TetraIndex(Index::from_raw_parts(value.gen, value.index))
    }
}

pub struct SearchData<D: Dimension> {
    pub point: Point<D>,
    pub radius: Float,
    pub tetra_index: TetraIndexSend,
}

pub struct SearchResult<D: Dimension> {
    pub point: Point<D>,
    /// The index in the Vec of the corresponding RadiusSearchData
    /// that produced this result.
    pub tetra_index: TetraIndexSend,
}

pub struct IndexedSearchResult<D: Dimension, I> {
    result: SearchResult<D>,
    point_index: I,
}

pub trait RadiusSearch<D: Dimension> {
    fn unique_radius_search(&mut self, data: Vec<SearchData<D>>) -> Vec<SearchResult<D>>;
}

pub trait IndexedRadiusSearch<D: Dimension> {
    type Index: PartialEq + Eq + Hash;
    fn radius_search(
        &mut self,
        data: Vec<SearchData<D>>,
    ) -> Vec<IndexedSearchResult<D, Self::Index>>;
}

pub struct HaloExporter<F, I> {
    radius_search: F,
    already_exported: StableHashSet<I>,
}

impl<F, I> HaloExporter<F, I> {
    fn new(radius_search: F) -> Self {
        Self {
            radius_search,
            already_exported: StableHashSet::default(),
        }
    }
}

impl<D: Dimension, F: IndexedRadiusSearch<D>> RadiusSearch<D> for HaloExporter<F, F::Index> {
    fn unique_radius_search(&mut self, data: Vec<SearchData<D>>) -> Vec<SearchResult<D>> {
        let indexed_results = self.radius_search.radius_search(data);
        indexed_results
            .into_iter()
            .filter_map(
                |IndexedSearchResult {
                     result,
                     point_index,
                 }| {
                    if self.already_exported.insert(point_index) {
                        Some(result)
                    } else {
                        None
                    }
                },
            )
            .collect()
    }
}

#[cfg(test)]
#[generic_tests::define]
mod tests {
    use super::IndexedRadiusSearch;
    use super::IndexedSearchResult;
    use super::SearchData;
    use super::SearchResult;
    use crate::prelude::ParticleId;
    use crate::test_utils::assert_float_is_close_high_error;
    use crate::voronoi::constructor::HaloIteration;
    use crate::voronoi::delaunay::halo_iteration::HaloExporter;
    use crate::voronoi::delaunay::Delaunay;
    use crate::voronoi::primitives::point::DVector;
    use crate::voronoi::test_utils::TestDimension;
    use crate::voronoi::utils::get_extent;
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

    pub struct LocalRadiusSearch<D: Dimension>(Vec<(ParticleId, Point<D>)>);

    impl<D: Dimension> IndexedRadiusSearch<D> for LocalRadiusSearch<D> {
        type Index = ParticleId;

        fn radius_search(
            &mut self,
            data: Vec<SearchData<D>>,
        ) -> Vec<IndexedSearchResult<D, Self::Index>> {
            data.iter()
                .flat_map(|data| {
                    self.0
                        .iter()
                        .filter(|(_, p)| data.point.distance(*p) < data.radius)
                        .map(move |(j, p)| IndexedSearchResult {
                            point_index: *j,
                            result: SearchResult {
                                point: *p,
                                tetra_index: data.tetra_index,
                            },
                        })
                })
                .collect()
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
        let (full_triangulation, full_map) =
            Triangulation::construct_from_iter(points.iter().cloned());
        // Now construct the triangulation of the first set using imported
        // halo particles imported from the other set.
        let extent = get_extent(points.iter().map(|(_, p)| p).cloned()).unwrap();
        let (sub_triangulation, sub_map) = HaloIteration::construct_from_iter(
            points1.iter().cloned(),
            HaloExporter::new(LocalRadiusSearch(points2)),
            extent,
        );
        let cons1 = TriangulationData::from_triangulation_and_map(full_triangulation, full_map);
        let cons2 = TriangulationData::from_triangulation_and_map(sub_triangulation, sub_map);
        let voronoi1 = cons1.construct_voronoi();
        let voronoi2 = cons2.construct_voronoi();
        for (id, _) in points1.iter() {
            let c1 = get_cell_for_particle(&voronoi1, &cons1, *id);
            let c2 = get_cell_for_particle(&voronoi2, &cons2, *id);
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
