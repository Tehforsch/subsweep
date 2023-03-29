mod halo_iteration;
mod local;

use bevy::utils::StableHashSet;

use self::halo_iteration::RadiusSearch;
pub(super) use self::halo_iteration::SearchData;
use self::halo_iteration::SearchResult;
use self::local::Local;
use super::delaunay::dimension::DTetra;
use super::delaunay::dimension::DTetraData;
use super::delaunay::PointIndex;
use super::delaunay::TetraIndex;
use super::primitives::DVector;
use super::utils::get_extent;
use super::Cell;
use super::CellIndex;
use super::Delaunay;
use super::DimensionCell;
use super::Point;
use super::Triangulation;
use super::TriangulationData;
use super::VoronoiGrid;
use crate::prelude::ParticleId;
use crate::voronoi::delaunay::PointKind;
use crate::voronoi::Dimension;

/// Determines by how much the search radius is increased beyond the
/// mathematically necessary radius (equal to the radius of the
/// circumcircle/sphere of the tetra), in order to prevent numerical
/// problems due to floating point arithmetic.
const SEARCH_SAFETY_FACTOR: f64 = 1.05;

pub struct Constructor<D: Dimension> {
    data: TriangulationData<D>,
}

impl<D> Constructor<D>
where
    D: Dimension,
    Triangulation<D>: Delaunay<D>,
    Cell<D>: DimensionCell<Dimension = D>,
{
    pub fn construct_from_iter<'b, F>(
        iter: impl Iterator<Item = (CellIndex, Point<D>)> + 'b,
        f: F,
    ) -> Self
    where
        F: RadiusSearch<D>,
    {
        let points: Vec<_> = iter.collect();
        let extent = f
            .determine_global_extent()
            .unwrap_or(get_extent(points.iter().map(|p| p.1)).unwrap());
        let (triangulation, map) =
            Triangulation::<D>::construct_from_iter_custom_extent(points.into_iter(), &extent);
        let mut iteration = HaloSearchIteration {
            triangulation,
            f,
            checked_tetras: StableHashSet::default(),
        };
        iteration.run();
        let data = TriangulationData::from_triangulation_and_map(iteration.triangulation, map);
        Self { data }
    }

    pub fn new(points: impl Iterator<Item = (CellIndex, Point<D>)>) -> Self {
        Self::construct_from_iter(points, Local)
    }

    pub fn only_delaunay<'a>(iter: impl Iterator<Item = &'a Point<D>> + 'a) -> Triangulation<D>
    where
        Point<D>: 'static,
    {
        Triangulation::construct_no_key(iter)
    }

    pub fn voronoi(&self) -> VoronoiGrid<D> {
        self.data.construct_voronoi()
    }

    pub fn get_point_by_particle_id(&self, particle_id: ParticleId) -> Option<PointIndex> {
        self.data
            .point_to_cell_map
            .get_by_left(&particle_id)
            .copied()
    }
}

struct HaloSearchIteration<D: Dimension, F> {
    triangulation: Triangulation<D>,
    f: F,
    checked_tetras: StableHashSet<TetraIndex>,
}

impl<D, F: RadiusSearch<D>> HaloSearchIteration<D, F>
where
    D: Dimension,
    Triangulation<D>: Delaunay<D>,
    F: RadiusSearch<D>,
    Cell<D>: DimensionCell<Dimension = D>,
{
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
                    self.triangulation.insert(point, PointKind::Halo);
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
                let tetra = &self.triangulation.tetras[t];
                let tetra_data = self.triangulation.get_tetra_data(tetra);
                let center = tetra_data.get_center_of_circumcircle();
                let sample_point = self.triangulation.points[tetra.points().next().unwrap()];
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
