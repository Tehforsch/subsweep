mod halo_iteration;
mod local;
pub(super) mod parallel;

pub use parallel::ParallelVoronoiGridConstruction;

use self::halo_iteration::HaloIteration;
use self::halo_iteration::RadiusSearch;
pub(super) use self::halo_iteration::SearchData;
use self::local::Local;
use super::delaunay::PointIndex;
use super::delaunay::TetraIndex;
use super::utils::get_extent;
use super::ActiveDimension;
use super::Cell;
use super::CellIndex;
use super::DCell;
use super::Delaunay;
use super::Point;
use super::Triangulation;
use super::TriangulationData;
use super::VoronoiGrid;
use crate::grid;
use crate::grid::FaceArea;
use crate::grid::ParticleType;
use crate::prelude::ParticleId;
use crate::units::Length;
use crate::units::VecDimensionless;
use crate::units::Volume;
use crate::voronoi::Dimension;

pub struct Constructor<D: Dimension> {
    data: TriangulationData<D>,
}

impl<D> Constructor<D>
where
    D: Dimension,
    Triangulation<D>: Delaunay<D>,
    Cell<D>: DCell<Dimension = D>,
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
        let (triangulation, mut map) =
            Triangulation::<D>::construct_from_iter_custom_extent(points.into_iter(), &extent);
        let mut iteration = HaloIteration::new(triangulation, search);
        iteration.run();
        map.extend(iteration.haloes);
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

    pub fn get_particle_id_by_point(&self, point_index: PointIndex) -> Option<ParticleId> {
        self.data
            .point_to_cell_map
            .get_by_right(&point_index)
            .copied()
    }

    pub fn get_point_by_particle_id(&self, particle_id: ParticleId) -> Option<PointIndex> {
        self.data
            .point_to_cell_map
            .get_by_left(&particle_id)
            .copied()
    }

    pub fn get_position_for_particle_id(&self, id: ParticleId) -> Point<D> {
        self.data.triangulation.points[self.get_point_by_particle_id(id).unwrap()]
    }
}

impl Constructor<ActiveDimension> {
    pub fn sweep_grid(&self) -> Vec<(CellIndex, ParticleType, grid::Cell)> {
        let voronoi = self.voronoi();
        voronoi
            .cells
            .iter()
            .map(|voronoi_cell| {
                let id = self
                    .get_particle_id_by_point(voronoi_cell.delaunay_point)
                    .unwrap();
                let neighbour_type = self.data.get_particle_type(voronoi_cell.delaunay_point);
                (
                    id,
                    neighbour_type,
                    grid::Cell {
                        neighbours: voronoi_cell
                            .faces
                            .iter()
                            .map(|face| {
                                (
                                    crate::grid::Face {
                                        area: FaceArea::new_unchecked(face.area),
                                        normal: VecDimensionless::new_unchecked(face.normal),
                                    },
                                    face.connection.clone(),
                                )
                            })
                            .collect(),
                        size: Length::new_unchecked(voronoi_cell.size()),
                        volume: Volume::new_unchecked(voronoi_cell.volume()),
                    },
                )
            })
            .collect()
    }
}
