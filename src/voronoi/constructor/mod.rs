mod halo_cache;
mod halo_iteration;
mod local;
pub(super) mod parallel;

use log::debug;
use log::info;
pub use parallel::ParallelVoronoiGridConstruction;

use self::halo_iteration::HaloIteration;
use self::halo_iteration::RadiusSearch;
pub(super) use self::halo_iteration::SearchData;
use self::local::Local;
use super::delaunay::PointIndex;
use super::delaunay::TetraIndex;
use super::visualizer::Visualizable;
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
use crate::extent::Extent;
use crate::hash_map::BiMap;
use crate::prelude::ParticleId;
use crate::sweep::grid;
use crate::sweep::grid::FaceArea;
use crate::sweep::grid::ParticleType;
use crate::units::Length;
use crate::units::VecDimensionless;
use crate::units::Volume;
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
    SearchData<D>: Visualizable,
    Extent<Point<D>>: Visualizable,
{
    pub fn construct_from_iter<'b, F>(
        iter: impl Iterator<Item = (ParticleId, Point<D>)> + 'b,
        mut search: F,
    ) -> Self
    where
        F: RadiusSearch<D>,
    {
        info!("Beginning local Delaunay construction.");
        let points: Vec<_> = iter.collect();
        let extent = search
            .determine_global_extent()
            .unwrap_or_else(|| Extent::from_points(points.iter().map(|p| p.1)).unwrap());
        let characteristic_length =
            get_characteristic_length::<D>(extent.max_side_length(), search.num_points());
        let extent = extent.including_periodic_images();
        let (triangulation, map) =
            Triangulation::<D>::construct_from_iter_custom_extent(points.into_iter(), &extent);
        let mut map: BiMap<_, _> = map
            .into_iter()
            .map(|(id, p)| (ParticleType::Local(id), p))
            .collect();
        info!("Finished local Delaunay construction, starting halo iteration.");
        let mut iteration = HaloIteration::new(triangulation, search, characteristic_length);
        iteration.run();
        map.extend(iteration.haloes);
        info!("Finished delaunay construction.",);
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

    pub fn iter_voronoi_cells(&self) -> impl Iterator<Item = Cell<D>> + '_ {
        self.data.iter_voronoi_cells()
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
        self.data
            .triangulation
            .get_original_point(self.get_point_by_cell(cell_index).unwrap())
    }
}

fn map_ptype(ptype: ParticleType, periodic: bool) -> ParticleType {
    if !periodic {
        match ptype {
            ParticleType::LocalPeriodic(_) => ParticleType::Boundary,
            ParticleType::RemotePeriodic(_) => ParticleType::Boundary,
            x => x,
        }
    } else {
        ptype
    }
}

impl Constructor<ActiveDimension> {
    pub fn sweep_grid(&self, periodic: bool) -> Vec<(ParticleType, grid::Cell)> {
        let voronoi_cells = self.iter_voronoi_cells();
        info!("Constructing sweep grid.");
        voronoi_cells
            .map(|voronoi_cell| {
                let particle_type = map_ptype(
                    self.data.get_particle_type(voronoi_cell.delaunay_point),
                    periodic,
                );
                (
                    particle_type,
                    grid::Cell {
                        neighbours: voronoi_cell
                            .faces
                            .iter()
                            .map(|face| {
                                (
                                    crate::sweep::grid::Face {
                                        area: FaceArea::new_unchecked(face.area),
                                        normal: VecDimensionless::new_unchecked(face.normal),
                                    },
                                    map_ptype(face.connection, periodic),
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
