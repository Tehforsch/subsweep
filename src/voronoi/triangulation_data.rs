use bevy::utils::StableHashMap;
use bimap::BiMap;

use super::delaunay::dimension::DTetra;
use super::delaunay::dimension::DTetraData;
use super::delaunay::dimension::Dimension;
use super::delaunay::Delaunay;
use super::delaunay::PointIndex;
use super::delaunay::PointKind;
use super::delaunay::TetraIndex;
use super::Cell;
use super::CellIndex;
use super::DCell;
use super::Point;
use super::Triangulation;
use super::VoronoiGrid;
use crate::grid::ParticleType;
use crate::grid::RemoteNeighbour;

pub struct TriangulationData<D: Dimension> {
    pub triangulation: Triangulation<D>,
    pub point_to_cell_map: BiMap<CellIndex, PointIndex>,
    pub point_to_tetras_map: StableHashMap<PointIndex, Vec<TetraIndex>>,
    pub tetra_to_voronoi_point_map: StableHashMap<TetraIndex, Point<D>>,
}

impl<D: Dimension> TriangulationData<D>
where
    Triangulation<D>: Delaunay<D>,
    Cell<D>: DCell<Dimension = D>,
{
    pub fn is_infinite_cell(&self, p: PointIndex) -> bool {
        let tetras = &self.point_to_tetras_map[&p];
        tetras.iter().any(|t| {
            self.triangulation.tetras[*t]
                .points()
                .any(|p| self.triangulation.point_kinds[&p] == PointKind::Outer)
        })
    }

    pub fn from_triangulation_and_map(
        t: Triangulation<D>,
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
                .iter_non_boundary_points()
                .map(|p| Cell::<D>::new(self, p))
                .collect(),
        }
    }

    pub fn get_particle_type(&self, p: PointIndex) -> ParticleType {
        use ParticleType::*;
        use PointKind::*;
        match self.triangulation.point_kinds[&p] {
            Inner => Local(*self.point_to_cell_map.get_by_right(&p).unwrap()),
            Outer => Boundary,
            Halo(rank) => Remote(RemoteNeighbour {
                id: *self.point_to_cell_map.get_by_right(&p).unwrap(),
                rank: rank,
            }),
        }
    }
}

fn point_to_tetra_map<D: Dimension>(
    triangulation: &Triangulation<D>,
) -> StableHashMap<PointIndex, Vec<TetraIndex>>
where
    D: Dimension,
    Triangulation<D>: Delaunay<D>,
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
