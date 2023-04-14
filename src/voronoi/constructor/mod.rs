mod halo_cache;
mod halo_iteration;
mod local;
pub(super) mod parallel;

use bevy::prelude::debug;
pub use parallel::ParallelVoronoiGridConstruction;

use self::halo_iteration::HaloIteration;
use self::halo_iteration::RadiusSearch;
pub(super) use self::halo_iteration::SearchData;
use self::local::Local;
use super::delaunay::PointIndex;
use super::delaunay::TetraIndex;
use super::Cell;
use super::CellIndex;
use super::DCell;
use super::Delaunay;
use super::Triangulation;
use super::TriangulationData;
use super::VoronoiGrid;
use crate::dimension::ActiveDimension;
use crate::dimension::ActiveWrapType;
use crate::dimension::Point;
use crate::extent::get_extent;
use crate::grid;
use crate::grid::FaceArea;
use crate::grid::ParticleType;
use crate::hash_map::BiMap;
use crate::prelude::ParticleId;
use crate::units::Length;
use crate::units::VecDimensionless;
use crate::units::Volume;
use crate::vis;
use crate::voronoi::constructor::halo_iteration::get_characteristic_length;
use crate::voronoi::DDimension;

pub struct Constructor<D: DDimension> {
    data: TriangulationData<D>,
}

impl<D> Constructor<D>
where
    D: DDimension<WrapType = ActiveWrapType>,
    Triangulation<D>: Delaunay<D>,
    Cell<D>: DCell<Dimension = D>,
{
    pub fn construct_from_iter<'b, F>(
        iter: impl Iterator<Item = (ParticleId, Point<D>)> + 'b,
        search: F,
    ) -> Self
    where
        F: RadiusSearch<D>,
    {
        debug!("Beginning local Delaunay construction.");
        let points: Vec<_> = iter.collect();
        let extent = search
            .determine_global_extent()
            .unwrap_or_else(|| get_extent(points.iter().map(|p| p.1)).unwrap());
        let characteristic_length =
            get_characteristic_length::<D>(extent.max_side_length(), points.len());
        let extent = extent.including_periodic_images();
        let (triangulation, map) =
            Triangulation::<D>::construct_from_iter_custom_extent(points.into_iter(), &extent);
        vis![&triangulation];
        let mut map: BiMap<_, _> = map
            .into_iter()
            .map(|(id, p)| (ParticleType::Local(id), p))
            .collect();
        debug!("Finished local Delaunay construction, starting halo iteration.");
        let mut iteration = HaloIteration::new(triangulation, search, characteristic_length);
        iteration.run();
        map.extend(iteration.haloes);
        debug!(
            "Finished delaunay construction of {} points ({} tetras).",
            iteration.triangulation.points.len(),
            iteration.triangulation.tetras.len()
        );
        let data = TriangulationData::from_triangulation_and_map(iteration.triangulation, map);
        Self { data }
    }

    pub fn new(points: impl Iterator<Item = (ParticleId, Point<D>)>) -> Self {
        Self::construct_from_iter(points, Local)
    }

    pub fn only_delaunay<'a>(iter: impl Iterator<Item = &'a Point<D>> + 'a) -> Triangulation<D>
    where
        Point<D>: 'static,
    {
        Triangulation::construct_no_key(iter)
    }

    pub fn voronoi(&self) -> VoronoiGrid<D> {
        debug!("Constructing voronoi grid.");
        self.data.construct_voronoi()
    }

    pub fn get_cell_by_point(&self, point_index: PointIndex) -> Option<ParticleType> {
        self.data
            .point_to_cell_map
            .get_by_right(&point_index)
            .cloned()
    }

    pub fn get_point_by_cell(&self, cell_index: CellIndex) -> Option<PointIndex> {
        self.data
            .point_to_cell_map
            .get_by_left(&cell_index)
            .copied()
    }

    pub fn get_position_for_cell(&self, cell_index: CellIndex) -> Point<D> {
        self.data.triangulation.points[self.get_point_by_cell(cell_index).unwrap()]
    }
}

impl Constructor<ActiveDimension> {
    pub fn sweep_grid(&self) -> Vec<(ParticleType, grid::Cell)> {
        let voronoi = self.voronoi();
        debug!("Constructing sweep grid.");
        voronoi
            .cells
            .iter()
            .map(|voronoi_cell| {
                let particle_type = self.data.get_particle_type(voronoi_cell.delaunay_point);
                (
                    particle_type,
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
