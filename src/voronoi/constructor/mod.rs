mod halo_iteration;
mod local;
pub(super) mod parallel;

use self::halo_iteration::HaloIteration;
use self::halo_iteration::RadiusSearch;
pub(super) use self::halo_iteration::SearchData;
use self::local::Local;
use super::delaunay::PointIndex;
use super::delaunay::TetraIndex;
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
use crate::voronoi::Dimension;

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
        search: F,
    ) -> Self
    where
        F: RadiusSearch<D>,
    {
        let points: Vec<_> = iter.collect();
        let extent = search
            .determine_global_extent()
            .unwrap_or_else(|| get_extent(points.iter().map(|p| p.1)).unwrap());
        let (triangulation, map) =
            Triangulation::<D>::construct_from_iter_custom_extent(points.into_iter(), &extent);
        let mut iteration = HaloIteration::new(triangulation, search);
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
