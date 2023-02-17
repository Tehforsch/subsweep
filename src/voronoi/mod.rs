mod delaunay;
mod face;
mod indexed_arena;
mod tetra;

use bevy::utils::StableHashMap;
pub use delaunay::DelaunayTriangulation;
use derive_more::From;
use derive_more::Into;
use generational_arena::Index;

use self::face::Face;
use self::indexed_arena::IndexedArena;
use self::tetra::Tetra;
use crate::grid::Cell;
use crate::grid::Neighbour;
use crate::prelude::ParticleId;
use crate::units::VecDimensionless;
use crate::units::VecLength;

#[derive(Debug, Clone, Copy, From, Into, PartialEq, Eq)]
pub struct TetraIndex(Index);
#[derive(Debug, Clone, Copy, From, Into, PartialEq, Eq)]
pub struct FaceIndex(Index);
#[derive(Debug, Clone, Copy, From, Into, PartialEq, Eq, Hash)]
pub struct PointIndex(Index);
#[derive(Debug, Clone, Copy, From, Into, PartialEq, Eq)]
pub struct CellIndex(Index);

#[cfg(feature = "2d")]
pub type Point = glam::DVec2;
#[cfg(feature = "3d")]
pub type Point = glam::DVec3;

fn point_to_length(p: Point) -> VecLength {
    VecLength::new_unchecked(p)
}

fn point_to_dimensionless(p: Point) -> VecDimensionless {
    VecDimensionless::new_unchecked(p)
}

type TetraList = IndexedArena<TetraIndex, Tetra>;
type FaceList = IndexedArena<FaceIndex, Face>;
type PointList = IndexedArena<PointIndex, Point>;
type CellList = IndexedArena<CellIndex, Cell>;

struct VoronoiGrid {
    cells: CellList,
}

impl From<DelaunayTriangulation> for VoronoiGrid {
    fn from(t: DelaunayTriangulation) -> Self {
        let mut map: StableHashMap<PointIndex, usize> = StableHashMap::default();
        let mut cells = vec![];
        for (i, (point_index, _)) in t.points.iter().enumerate() {
            cells.push(Cell {
                neighbours: vec![],
                size: todo!(),
            });
            map.insert(point_index, i);
        }
        for (_, face) in t.faces.iter() {
            let i1 = map[&face.p1];
            let i2 = map[&face.p2];
            let normal =
                point_to_dimensionless((t.points[face.p1] - t.points[face.p2]).normalize());
            let face12 = crate::grid::Face {
                area: todo!(),
                normal,
            };
            let face21 = crate::grid::Face {
                area: todo!(),
                normal: -normal,
            };
            cells[i1]
                .neighbours
                .push((face12, Neighbour::Local(ParticleId(i2))));
            cells[i2]
                .neighbours
                .push((face21, Neighbour::Local(ParticleId(i1))));
        }
        todo!()
    }
}
