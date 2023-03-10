use bevy::prelude::Entity;
use bevy::utils::hashbrown::HashMap;

use super::delaunay::dimension::Dimension;
use super::delaunay::Delaunay;
use super::DelaunayTriangulation;
use super::PointIndex;
use super::VoronoiGrid;
use crate::grid::Cell;
use crate::grid::FaceArea;
use crate::grid::Neighbour;
use crate::prelude::MVec;
use crate::prelude::ParticleId;
use crate::units::Length;
use crate::units::VecDimensionless;
use crate::units::Volume;

pub fn construct_grid_from_iter<D>(
    iter: impl Iterator<Item = (Entity, <D as Dimension>::Point)>,
) -> Vec<(Entity, ParticleId, Cell)>
where
    D: Dimension<Point = MVec>,
    DelaunayTriangulation<D>: Delaunay<D>,
    VoronoiGrid<D>: for<'a> From<&'a DelaunayTriangulation<D>>,
{
    let mut entities = vec![];
    let mut positions = vec![];
    for (entity, pos) in iter {
        entities.push(entity);
        positions.push(pos);
    }
    let (triangulation, indices) = DelaunayTriangulation::<D>::construct(&positions);
    let point_index_to_entity: HashMap<PointIndex, Entity> = entities
        .iter()
        .enumerate()
        .map(|(i, entity)| (indices[i], *entity))
        .collect();
    let grid = VoronoiGrid::from(&triangulation);
    grid.cells
        .iter()
        .filter_map(|voronoi_cell| {
            let entity = point_index_to_entity.get(&voronoi_cell.delaunay_point);
            entity.map(|entity| {
                let id = ParticleId(voronoi_cell.index);
                let grid_cell = crate::grid::Cell {
                    neighbours: voronoi_cell
                        .iter_neighbours_and_faces(&grid)
                        .map(|(neigh, area, normal)| {
                            let face = crate::grid::Face {
                                area: FaceArea::new_unchecked(area),
                                normal: VecDimensionless::new_unchecked(normal),
                            };
                            if grid.cells[neigh].is_boundary {
                                (face, Neighbour::Boundary)
                            } else {
                                (face, Neighbour::Local(ParticleId(neigh)))
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
#[generic_tests::define]
mod tests {
    use bevy::prelude::Entity;

    use super::super::delaunay::dimension::Dimension;
    use super::super::primitives::Point2d;
    use super::super::primitives::Point3d;
    use super::construct_grid_from_iter;
    use crate::voronoi::ActiveDimension;
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

    #[test]
    fn construct_small_grid() {
        construct_grid_from_iter::<ActiveDimension>(
            ActiveDimension::get_points_for_small_grid()
                .into_iter()
                .enumerate()
                .map(move |(i, p)| (Entity::from_raw(i as u32), p.clone())),
        );
    }
}
