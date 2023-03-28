use std::hash::Hash;

use bevy::utils::StableHashSet;
use bimap::BiMap;
use generational_arena::Index;
use mpi::traits::Equivalence;

use super::Delaunay;
use super::DelaunayTriangulation;
use super::Point;
use super::PointIndex;
use super::PointKind;
use super::TetraIndex;
use crate::voronoi::delaunay::dimension::DimensionTetra;
use crate::voronoi::delaunay::dimension::DimensionTetraData;
use crate::voronoi::primitives::point::Vector;
use crate::voronoi::primitives::Float;
use crate::voronoi::utils::Extent;
use crate::voronoi::Dimension;

/// Determines by how much the search radius is increased beyond the
/// mathematically necessary radius (equal to the radius of the
/// circumcircle/sphere of the tetra), in order to prevent numerical
/// problems due to floating point arithmetic.
const SEARCH_SAFETY_FACTOR: f64 = 1.05;

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
    point: Point<D>,
    radius: Float,
    tetra_index: TetraIndexSend,
}

pub struct SearchResult<D: Dimension> {
    point: Point<D>,
    /// The index in the Vec of the corresponding RadiusSearchData
    /// that produced this result.
    tetra_index: TetraIndexSend,
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

pub struct GhostExporter<F, I> {
    radius_search: F,
    already_exported: StableHashSet<I>,
}

impl<F, I> GhostExporter<F, I> {
    fn new(radius_search: F) -> Self {
        Self {
            radius_search,
            already_exported: StableHashSet::default(),
        }
    }
}

impl<D: Dimension, F: IndexedRadiusSearch<D>> RadiusSearch<D> for GhostExporter<F, F::Index> {
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

pub struct GhostIteration<'a, D: Dimension, F: RadiusSearch<D>> {
    tri: &'a mut DelaunayTriangulation<D>,
    f: F,
    checked_tetras: StableHashSet<TetraIndex>,
}

impl<'a, D, F> GhostIteration<'a, D, F>
where
    D: Dimension + 'a,
    DelaunayTriangulation<D>: Delaunay<D>,
    F: RadiusSearch<D>,
{
    pub fn construct_from_iter<'b, T: Hash + Clone + Eq>(
        iter: impl Iterator<Item = (T, Point<D>)> + 'b,
        f: F,
        extent: Extent<Point<D>>,
    ) -> (DelaunayTriangulation<D>, BiMap<T, PointIndex>) {
        let (mut tri, map) =
            DelaunayTriangulation::<D>::construct_from_iter_custom_extent(iter, &extent);
        {
            let mut iteration = GhostIteration {
                tri: &mut tri,
                f,
                checked_tetras: StableHashSet::default(),
            };
            iteration.run();
        }
        (tri, map)
    }

    fn run(&mut self) {
        while self.iter_remaining_tetras().next().is_some() {
            self.iterate();
        }
    }

    fn iterate(&mut self) {
        let search_data = self.get_radius_search_data();
        let newly_imported = self.f.unique_radius_search(search_data);
        println!("Imported: {:>8}", newly_imported.len());
        let checked: StableHashSet<TetraIndex> = self.iter_remaining_tetras().collect();
        let tetras_with_new_points_in_vicinity = newly_imported
            .into_iter()
            .map(
                |SearchResult {
                     point,
                     tetra_index: search_index,
                 }| {
                    self.tri.insert(point, PointKind::Ghost);
                    search_index.into()
                },
            )
            .collect();
        self.checked_tetras
            .extend(checked.difference(&tetras_with_new_points_in_vicinity));
    }

    fn get_radius_search_data(&self) -> Vec<SearchData<D>> {
        self.iter_remaining_tetras()
            .map(|t| {
                let tetra = &self.tri.tetras[t];
                let tetra_data = self.tri.get_tetra_data(tetra);
                let center = tetra_data.get_center_of_circumcircle();
                let sample_point = self.tri.points[tetra.points().next().unwrap()];
                let radius_circumcircle = center.distance(sample_point);
                let radius = SEARCH_SAFETY_FACTOR * radius_circumcircle;
                SearchData::<D> {
                    radius,
                    point: center,
                    tetra_index: t.into(),
                }
            })
            .collect()
    }

    fn iter_remaining_tetras(&self) -> impl Iterator<Item = TetraIndex> + '_ {
        self.tri
            .tetras
            .iter()
            .map(|(i, _)| i)
            .filter(|i| !self.checked_tetras.contains(&i))
    }
}

#[cfg(test)]
#[generic_tests::define]
mod tests {
    use super::GhostIteration;
    use super::IndexedRadiusSearch;
    use super::IndexedSearchResult;
    use super::SearchData;
    use super::SearchResult;
    use crate::prelude::ParticleId;
    use crate::test_utils::assert_float_is_close_high_error;
    use crate::voronoi::delaunay::ghost_iteration::GhostExporter;
    use crate::voronoi::delaunay::tests::TestableDimension;
    use crate::voronoi::delaunay::Delaunay;
    use crate::voronoi::primitives::point::Vector;
    use crate::voronoi::utils::get_extent;
    use crate::voronoi::Cell;
    use crate::voronoi::Constructor;
    use crate::voronoi::DelaunayTriangulation;
    use crate::voronoi::Dimension;
    use crate::voronoi::DimensionCell;
    use crate::voronoi::Point;
    use crate::voronoi::ThreeD;
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
        cons: &'a Constructor<D>,
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
    pub fn voronoi_grid_with_ghost_points_is_the_same_as_without<D>()
    where
        D: Dimension + TestableDimension,
        DelaunayTriangulation<D>: Delaunay<D>,
        Point<D>: Vector,
        Cell<D>: DimensionCell<Dimension = D>,
    {
        let points: Vec<_> = D::get_example_point_set()
            .into_iter()
            .enumerate()
            .map(|(i, p)| (ParticleId(i as u64), p))
            .collect();
        // First construct the triangulation normally
        let (triangulation1, map1) =
            DelaunayTriangulation::construct_from_iter(points.iter().cloned());
        // Now split the point set on some arbitrary criterion and
        // construct the sub-triangulation of the first set using imported
        // ghosts of the other set.
        let half_len = points.len() / 2;
        let extent = get_extent(points.iter().map(|(_, p)| p).cloned()).unwrap();
        let (points_1, points_2) = points.split_at(half_len);
        let points_2 = points_2.iter().cloned().collect();
        let (triangulation2, map2) = GhostIteration::construct_from_iter(
            points_1.into_iter().cloned(),
            GhostExporter::new(LocalRadiusSearch(points_2)),
            extent,
        );
        let cons1 = Constructor::from_triangulation_and_map(triangulation1, map1);
        let cons2 = Constructor::from_triangulation_and_map(triangulation2, map2);
        let voronoi1 = cons1.construct_voronoi();
        let voronoi2 = cons2.construct_voronoi();
        for (id, _) in points_1.iter() {
            let c1 = get_cell_for_particle(&voronoi1, &cons1, *id);
            let c2 = get_cell_for_particle(&voronoi2, &cons2, *id);
            assert_eq!(c1.faces.len(), c2.faces.len());
            assert_float_is_close_high_error(c1.volume(), c2.volume());
        }
    }
}
