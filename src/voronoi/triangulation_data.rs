use super::delaunay::dimension::DDimension;
use super::delaunay::dimension::DTetra;
use super::delaunay::dimension::DTetraData;
use super::delaunay::Delaunay;
use super::delaunay::PointIndex;
use super::delaunay::PointKind;
use super::delaunay::TetraIndex;
use super::Cell;
use super::CellIndex;
use super::DCell;
use super::Triangulation;
use super::VoronoiGrid;
use crate::dimension::Point;
use crate::grid::ParticleType;
use crate::hash_map::BiMap;
use crate::hash_map::HashMap;

pub struct TriangulationData<D: DDimension> {
    pub triangulation: Triangulation<D>,
    pub point_to_cell_map: BiMap<CellIndex, PointIndex>,
    pub point_to_tetras_map: HashMap<PointIndex, Vec<TetraIndex>>,
    pub tetra_to_voronoi_point_map: HashMap<TetraIndex, Point<D>>,
}

impl<D: DDimension> TriangulationData<D>
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
        point_to_cell_map: BiMap<ParticleType, PointIndex>,
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
            point_to_cell_map,
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
        if self.triangulation.point_kinds[&p] == PointKind::Outer {
            return ParticleType::Boundary;
        }
        *self.point_to_cell_map.get_by_right(&p).unwrap()
    }
}

fn point_to_tetra_map<D: DDimension>(
    triangulation: &Triangulation<D>,
) -> HashMap<PointIndex, Vec<TetraIndex>>
where
    D: DDimension,
    Triangulation<D>: Delaunay<D>,
{
    let mut map: HashMap<_, _> = triangulation
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
