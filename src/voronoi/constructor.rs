use std::hash::Hash;

use bevy::utils::StableHashSet;
use bimap::BiMap;

use super::delaunay::halo_iteration::RadiusSearch;
use super::delaunay::halo_iteration::SearchData;
use super::delaunay::TetraIndex;
use super::Delaunay;
use super::Point;
use super::PointIndex;
use super::Triangulation;
use crate::voronoi::delaunay::dimension::DTetra;
use crate::voronoi::delaunay::dimension::DTetraData;
use crate::voronoi::delaunay::halo_iteration::SearchResult;
use crate::voronoi::delaunay::PointKind;
use crate::voronoi::primitives::point::DVector;
use crate::voronoi::utils::Extent;
use crate::voronoi::Dimension;

/// Determines by how much the search radius is increased beyond the
/// mathematically necessary radius (equal to the radius of the
/// circumcircle/sphere of the tetra), in order to prevent numerical
/// problems due to floating point arithmetic.
const SEARCH_SAFETY_FACTOR: f64 = 1.05;

pub struct HaloIteration<D: Dimension, F: RadiusSearch<D>> {
    tri: Triangulation<D>,
    f: F,
    checked_tetras: StableHashSet<TetraIndex>,
}

impl<D, F> HaloIteration<D, F>
where
    D: Dimension,
    Triangulation<D>: Delaunay<D>,
    F: RadiusSearch<D>,
{
    pub fn construct_from_iter<'b, T: Hash + Clone + Eq>(
        iter: impl Iterator<Item = (T, Point<D>)> + 'b,
        f: F,
        extent: Extent<Point<D>>,
    ) -> (Triangulation<D>, BiMap<T, PointIndex>) {
        let (tri, map) = Triangulation::<D>::construct_from_iter_custom_extent(iter, &extent);
        let mut iteration = HaloIteration {
            tri: tri,
            f,
            checked_tetras: StableHashSet::default(),
        };
        iteration.run();
        (iteration.tri, map)
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
