use bevy::prelude::Entity;
use bevy::utils::StableHashMap;
use bimap::BiMap;

use super::delaunay::dimension::Dimension;
use super::delaunay::dimension::DimensionTetra;
use super::delaunay::dimension::DimensionTetraData;
use super::delaunay::Delaunay;
use super::delaunay::PointIndex;
use super::delaunay::PointKind;
use super::delaunay::TetraIndex;
use super::visualizer::Visualizable;
use super::Cell;
use super::CellIndex;
use super::DelaunayTriangulation;
use super::DimensionCell;
use super::Point;
use super::VoronoiGrid;
use crate::grid;
use crate::grid::FaceArea;
use crate::grid::ParticleType;
use crate::prelude::MVec;
use crate::prelude::ParticleId;
use crate::units::Length;
use crate::units::VecDimensionless;
use crate::units::Volume;
use crate::voronoi::cell::CellConnection;

pub struct TriangulationData<D: Dimension> {
    pub triangulation: DelaunayTriangulation<D>,
    pub point_to_cell_map: BiMap<CellIndex, PointIndex>,
    pub point_to_tetras_map: StableHashMap<PointIndex, Vec<TetraIndex>>,
    pub tetra_to_voronoi_point_map: StableHashMap<TetraIndex, Point<D>>,
}

impl<D: Dimension> TriangulationData<D>
where
    DelaunayTriangulation<D>: Delaunay<D>,
    Cell<D>: DimensionCell<Dimension = D>,
{
    pub fn new(points: impl Iterator<Item = (CellIndex, Point<D>)>) -> Self {
        let (t, map) = DelaunayTriangulation::construct_from_iter(points);
        Self::from_triangulation_and_map(t, map)
    }

    pub fn is_infinite_cell(&self, p: PointIndex) -> bool {
        let tetras = &self.point_to_tetras_map[&p];
        tetras.iter().any(|t| {
            self.triangulation.tetras[*t]
                .points()
                .any(|p| self.triangulation.point_kinds[&p] == PointKind::Outer)
        })
    }

    pub fn from_triangulation_and_map(
        t: DelaunayTriangulation<D>,
        map: BiMap<CellIndex, PointIndex>,
    ) -> Self {
        let tetra_to_voronoi_point_map = t
            .tetras
            .iter()
            .map(|(i, tetra)| (i, t.get_tetra_data(tetra).get_center_of_circumcircle()))
            .collect();
        let point_to_tetras_map = point_to_tetra_map(&t);
        Self {
            triangulation: t,
            point_to_tetras_map,
            point_to_cell_map: map,
            tetra_to_voronoi_point_map,
        }
    }

    pub fn construct_voronoi(&self) -> VoronoiGrid<D> {
        VoronoiGrid {
            cells: self
                .triangulation
                .iter_inner_points()
                .map(|p| Cell::<D>::new(self, p))
                .collect(),
        }
    }

    pub fn get_connection(&self, p: PointIndex) -> CellConnection {
        self.point_to_cell_map
            .get_by_right(&p)
            .map(|i| CellConnection::ToInner(*i))
            .unwrap_or(CellConnection::ToOuter)
    }
}

fn point_to_tetra_map<D: Dimension>(
    triangulation: &DelaunayTriangulation<D>,
) -> StableHashMap<PointIndex, Vec<TetraIndex>>
where
    D: Dimension,
    DelaunayTriangulation<D>: Delaunay<D>,
{
    let mut map: StableHashMap<_, _> = triangulation
        .points
        .iter()
        .map(|(i, _)| (i, vec![]))
        .collect();
    for (tetra_index, tetra) in triangulation.tetras.iter() {
        for p in tetra.points() {
            map.get_mut(&p).unwrap().push(tetra_index);
        }
    }
    map
}

pub fn construct_grid_from_iter<D>(
    iter: impl Iterator<Item = (Entity, ParticleId, <D as Dimension>::Point)>,
) -> Vec<(Entity, grid::Cell)>
where
    D: Dimension<Point = MVec>,
    DelaunayTriangulation<D>: Delaunay<D>,
    Cell<D>: DimensionCell<Dimension = D>,
    <Cell<D> as DimensionCell>::Dimension: Dimension<Point = MVec>,
    VoronoiGrid<D>: for<'a> From<&'a TriangulationData<D>>,
    <D as Dimension>::TetraData: Visualizable,
    <D as Dimension>::Point: Visualizable,
{
    let (triangulation, map) = DelaunayTriangulation::<D>::construct_from_iter(
        iter.map(|(entity, id, point)| ((id, entity), point)),
    );
    let id_to_point_index = map.iter().map(|((i, _), point)| (*i, *point)).collect();
    let entity_to_point_index: BiMap<_, _> = map
        .iter()
        .map(|((_, entity), point)| (*entity, *point))
        .collect();
    let cons = TriangulationData::from_triangulation_and_map(triangulation, id_to_point_index);
    let grid = cons.construct_voronoi();
    grid.cells
        .iter()
        .filter_map(|voronoi_cell| {
            let entity = entity_to_point_index.get_by_right(&voronoi_cell.delaunay_point);
            entity.map(|entity| {
                let grid_cell = grid::Cell {
                    neighbours: voronoi_cell
                        .faces
                        .iter()
                        .map(|face| {
                            let neigh = face.connection;
                            let face = crate::grid::Face {
                                area: FaceArea::new_unchecked(face.area),
                                normal: VecDimensionless::new_unchecked(face.normal),
                            };
                            if let CellConnection::ToInner(neigh) = neigh {
                                (face, ParticleType::Local(neigh))
                            } else {
                                (face, ParticleType::Boundary)
                            }
                        })
                        .collect(),
                    size: Length::new_unchecked(voronoi_cell.size()),
                    volume: Volume::new_unchecked(voronoi_cell.volume()),
                };
                (*entity, grid_cell)
            })
        })
        .collect()
}
