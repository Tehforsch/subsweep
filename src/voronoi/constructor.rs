use bevy::prelude::Entity;
use bevy::utils::StableHashMap;
use bimap::BiMap;

use super::delaunay::dimension::Dimension;
use super::delaunay::dimension::DimensionTetra;
use super::delaunay::dimension::DimensionTetraData;
use super::delaunay::Delaunay;
use super::delaunay::PointIndex;
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
use crate::grid::Neighbour;
use crate::prelude::MVec;
use crate::prelude::ParticleId;
use crate::units::Length;
use crate::units::VecDimensionless;
use crate::units::Volume;
use crate::voronoi::cell::CellConnection;

pub struct Constructor<D: Dimension> {
    pub triangulation: DelaunayTriangulation<D>,
    pub point_to_cell_map: BiMap<CellIndex, PointIndex>,
    pub point_to_tetras_map: StableHashMap<PointIndex, Vec<TetraIndex>>,
    pub tetra_to_voronoi_point_map: StableHashMap<TetraIndex, Point<D>>,
}

impl<D: Dimension> Constructor<D>
where
    DelaunayTriangulation<D>: Delaunay<D>,
    Cell<D>: DimensionCell<Dimension = D>,
{
    pub fn new(t: DelaunayTriangulation<D>, map: BiMap<CellIndex, PointIndex>) -> Self {
        let tetra_to_voronoi_point_map = t
            .tetras
            .iter()
            .map(|(i, tetra)| (i, t.get_tetra_data(&tetra).get_center_of_circumcircle()))
            .collect();
        let point_to_tetras_map = point_to_tetra_map(&t);
        Self {
            triangulation: t,
            point_to_tetras_map,
            point_to_cell_map: map,
            tetra_to_voronoi_point_map,
        }
    }

    pub fn construct(&self) -> VoronoiGrid<D> {
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
    iter: impl Iterator<Item = (Entity, <D as Dimension>::Point)>,
) -> Vec<(Entity, ParticleId, grid::Cell)>
where
    D: Dimension<Point = MVec>,
    DelaunayTriangulation<D>: Delaunay<D>,
    Cell<D>: DimensionCell<Dimension = D>,
    <Cell<D> as DimensionCell>::Dimension: Dimension<Point = MVec>,
    VoronoiGrid<D>: for<'a> From<&'a Constructor<D>>,
    <D as Dimension>::TetraData: Visualizable,
    <D as Dimension>::Point: Visualizable,
{
    let (triangulation, map) = DelaunayTriangulation::<D>::construct_from_iter(
        iter.enumerate()
            .map(|(i, (entity, point))| ((i, entity), point)),
    );
    let cell_index_to_point_index = map.iter().map(|((i, _), point)| (*i, *point)).collect();
    let entity_to_point_index: BiMap<_, _> = map
        .iter()
        .map(|((_, entity), point)| (*entity, *point))
        .collect();
    let cons = Constructor::new(triangulation, cell_index_to_point_index);
    let grid = VoronoiGrid::from(&cons);
    grid.cells
        .iter()
        .filter_map(|voronoi_cell| {
            let entity = entity_to_point_index.get_by_right(&voronoi_cell.delaunay_point);
            entity.map(|entity| {
                let id = ParticleId(voronoi_cell.index);
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
                                (face, Neighbour::Local(ParticleId(neigh)))
                            } else {
                                (face, Neighbour::Boundary)
                            }
                        })
                        .collect(),
                    size: Length::new_unchecked(voronoi_cell.size()),
                    volume: Volume::new_unchecked(voronoi_cell.volume()),
                };
                (*entity, id, grid_cell)
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::delaunay::dimension::Dimension;
    use super::super::primitives::Point2d;
    use super::super::primitives::Point3d;
    use crate::voronoi::ThreeD;
    use crate::voronoi::TwoD;

    trait TestableDimension: Dimension {
        fn get_points_for_small_grid() -> Vec<<Self as Dimension>::Point>;
    }

    impl TestableDimension for TwoD {
        fn get_points_for_small_grid() -> Vec<Point2d> {
            vec![
                Point2d::new(0.25, 0.25),
                Point2d::new(0.5, 0.5),
                Point2d::new(0.5, 0.25),
                Point2d::new(0.125, 0.5),
                Point2d::new(0.5, 0.125),
                Point2d::new(0.8, 0.1),
                Point2d::new(0.1, 0.8),
            ]
        }
    }

    impl TestableDimension for ThreeD {
        fn get_points_for_small_grid() -> Vec<Point3d> {
            vec![
                Point3d::new(0.5, 0.5, 0.5),
                Point3d::new(0.25, 0.55, 0.3),
                Point3d::new(0.5, 0.25, 0.4),
                Point3d::new(0.125, 0.53, 0.2),
                Point3d::new(0.8, 0.1, 0.23),
                Point3d::new(0.1, 0.8, 0.7),
            ]
        }
    }

    #[cfg(feature = "2d")]
    #[test]
    fn construct_small_grid_2d() {
        use bevy::prelude::Entity;

        use super::construct_grid_from_iter;

        construct_grid_from_iter::<TwoD>(
            TwoD::get_points_for_small_grid()
                .into_iter()
                .enumerate()
                .map(move |(i, p)| (Entity::from_raw(i as u32), p.clone())),
        );
    }
}
