use std::hash::Hash;

use bevy::utils::StableHashSet;
use bimap::BiMap;
use generational_arena::Index;
use mpi::traits::Equivalence;

use super::Delaunay;
use super::Point;
use super::PointIndex;
use super::PointKind;
use super::TetraIndex;
use super::Triangulation;
use crate::voronoi::delaunay::dimension::DTetra;
use crate::voronoi::delaunay::dimension::DTetraData;
use crate::voronoi::primitives::point::DVector;
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
    pub point: Point<D>,
    pub radius: Float,
    pub tetra_index: TetraIndexSend,
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

pub struct HaloIteration<'a, D: Dimension, F: RadiusSearch<D>> {
    tri: &'a mut Triangulation<D>,
    f: F,
    checked_tetras: StableHashSet<TetraIndex>,
}

impl<'a, D, F> HaloIteration<'a, D, F>
where
    D: Dimension + 'a,
    Triangulation<D>: Delaunay<D>,
    F: RadiusSearch<D>,
{
    pub fn construct_from_iter<'b, T: Hash + Clone + Eq>(
        iter: impl Iterator<Item = (T, Point<D>)> + 'b,
        f: F,
        extent: Extent<Point<D>>,
    ) -> (Triangulation<D>, BiMap<T, PointIndex>) {
        let (mut tri, map) = Triangulation::<D>::construct_from_iter_custom_extent(iter, &extent);
        {
            let mut iteration = HaloIteration {
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
        let checked: StableHashSet<TetraIndex> = self.iter_remaining_tetras().collect();
        println!(
            "To check: {:>8}, Imported: {:>8}",
            checked.len(),
            newly_imported.len()
        );
        let tetras_with_new_points_in_vicinity: StableHashSet<_> = newly_imported
            .into_iter()
            .map(
                |SearchResult {
                     point,
                     tetra_index: search_index,
                 }| {
                    self.tri.insert(point, PointKind::Halo);
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

    fn tetra_should_be_checked(&self, t: TetraIndex) -> bool {
        let tetra = &self.tri.tetras[t];
        tetra
            .points()
            .any(|p| self.tri.point_kinds[&p] == PointKind::Inner)
            && tetra
                .points()
                .all(|p| self.tri.point_kinds[&p] != PointKind::Outer)
    }

    fn iter_remaining_tetras(&self) -> impl Iterator<Item = TetraIndex> + '_ {
        self.tri
            .tetras
            .iter()
            .map(|(t, _)| t)
            .filter(|t| !self.checked_tetras.contains(t) && self.tetra_should_be_checked(*t))
    }
}

#[cfg(test)]
#[generic_tests::define]
mod tests {
    use super::HaloIteration;
    use super::IndexedRadiusSearch;
    use super::IndexedSearchResult;
    use super::SearchData;
    use super::SearchResult;
    use crate::prelude::ParticleId;
    use crate::test_utils::assert_float_is_close_high_error;
    use crate::voronoi::delaunay::halo_iteration::HaloExporter;
    use crate::voronoi::delaunay::Delaunay;
    use crate::voronoi::primitives::point::DVector;
    use crate::voronoi::utils::get_extent;
    use crate::voronoi::Cell;
    use crate::voronoi::Dimension;
    use crate::voronoi::DimensionCell;
    use crate::voronoi::Point;
    use crate::voronoi::Point2d;
    use crate::voronoi::Point3d;
    use crate::voronoi::ThreeD;
    use crate::voronoi::Triangulation;
    use crate::voronoi::TriangulationData;
    use crate::voronoi::TwoD;
    use crate::voronoi::VoronoiGrid;

    #[instantiate_tests(<TwoD>)]
    mod two_d {}

    #[instantiate_tests(<ThreeD>)]
    mod three_d {}

    pub trait TestableDimension: Dimension {
        fn get_example_point_sets() -> (Vec<Self::Point>, Vec<Self::Point>);

        fn get_combined_point_set() -> Vec<(ParticleId, Self::Point)> {
            let (p1, p2) = Self::get_example_point_sets_with_ids();
            p1.into_iter().chain(p2.into_iter()).collect()
        }

        fn get_example_point_sets_with_ids() -> (
            Vec<(ParticleId, Self::Point)>,
            Vec<(ParticleId, Self::Point)>,
        ) {
            let (p1, p2) = Self::get_example_point_sets();
            let len_p1 = p1.len();
            (
                p1.into_iter()
                    .enumerate()
                    .map(|(i, p)| (ParticleId(i as u64), p))
                    .collect(),
                p2.into_iter()
                    .enumerate()
                    .map(|(i, p)| (ParticleId(len_p1 as u64 + i as u64), p))
                    .collect(),
            )
        }
    }

    impl TestableDimension for TwoD {
        fn get_example_point_sets() -> (Vec<Self::Point>, Vec<Self::Point>) {
            use rand::Rng;
            use rand::SeedableRng;
            let mut rng = rand::rngs::StdRng::seed_from_u64(1338);
            let p1 = (0..100)
                .map(|_| {
                    let x = rng.gen_range(0.1..0.4);
                    let y = rng.gen_range(0.1..0.4);
                    Point2d::new(x, y)
                })
                .collect();
            let p2 = (0..100)
                .map(|_| {
                    let x = rng.gen_range(0.4..0.7);
                    let y = rng.gen_range(0.1..0.4);
                    Point2d::new(x, y)
                })
                .collect();
            (p1, p2)
        }
    }

    impl TestableDimension for ThreeD {
        fn get_example_point_sets() -> (Vec<Self::Point>, Vec<Self::Point>) {
            use rand::Rng;
            use rand::SeedableRng;
            let mut rng = rand::rngs::StdRng::seed_from_u64(1338);
            let p1 = (0..100)
                .map(|_| {
                    let x = rng.gen_range(0.1..0.4);
                    let y = rng.gen_range(0.1..0.4);
                    let z = rng.gen_range(0.1..0.4);
                    Point3d::new(x, y, z)
                })
                .collect();
            let p2 = (0..100)
                .map(|_| {
                    let x = rng.gen_range(0.4..0.7);
                    let y = rng.gen_range(0.1..0.4);
                    let z = rng.gen_range(0.1..0.4);
                    Point3d::new(x, y, z)
                })
                .collect();
            (p1, p2)
        }
    }

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
        D: Dimension + TestableDimension,
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
