mod delaunay;
mod face;
mod indexed_arena;
mod tetra;

use std::iter;

use bevy::utils::StableHashMap;
pub use delaunay::DelaunayTriangulation;
use derive_more::From;
use derive_more::Into;
use generational_arena::Index;

use self::face::Face;
use self::indexed_arena::IndexedArena;
use self::tetra::Tetra;

#[derive(Debug, Clone, Copy, From, Into, PartialEq, Eq)]
pub struct TetraIndex(Index);
#[derive(Debug, Clone, Copy, From, Into, PartialEq, Eq)]
pub struct FaceIndex(Index);
#[derive(Debug, Clone, Copy, From, Into, PartialEq, Eq, Hash)]
pub struct PointIndex(Index);

pub type CellIndex = usize;

#[cfg(feature = "2d")]
pub type Point = glam::DVec2;
#[cfg(feature = "3d")]
pub type Point = glam::DVec3;

type TetraList = IndexedArena<TetraIndex, Tetra>;
type FaceList = IndexedArena<FaceIndex, Face>;
type PointList = IndexedArena<PointIndex, Point>;

pub struct VoronoiGrid {
    pub cells: Vec<Cell>,
}

pub struct Cell {
    pub points: Vec<Point>,
    pub connected_cells: Vec<CellIndex>,
}

impl From<DelaunayTriangulation> for VoronoiGrid {
    fn from(t: DelaunayTriangulation) -> Self {
        let mut map: StableHashMap<PointIndex, CellIndex> = StableHashMap::default();
        let point_to_tetra_map = point_to_tetra_map(&t);
        let mut cells = vec![];
        for (i, (point_index, _)) in t.points.iter().enumerate() {
            map.insert(point_index, i);
        }
        for (point_index, _) in t.points.iter() {
            let mut points = vec![];
            let mut connected_cells = vec![];
            let tetras = &point_to_tetra_map[&point_index];
            for tetra in tetras.iter() {
                points.push(
                    t.get_tetra_data(&t.tetras[*tetra])
                        .get_center_of_circumcircle(),
                );
            }
            for (t1, t2) in tetras
                .iter()
                .zip(tetras[1..].iter().chain(iter::once(&tetras[0])))
            {
                let common_face = t.tetras[*t1].get_common_face_with(&t.tetras[*t2]);
                let other_point = t.faces[common_face].get_other_point(point_index);
                connected_cells.push(map[&other_point]);
            }
            cells.push(Cell {
                points,
                connected_cells,
            });
        }
        VoronoiGrid { cells }
    }
}

fn point_to_tetra_map(t: &DelaunayTriangulation) -> StableHashMap<PointIndex, Vec<TetraIndex>> {
    let mut map: StableHashMap<_, _> = t.points.iter().map(|(i, _)| (i, vec![])).collect();
    for (tetra_index, tetra) in t.tetras.iter() {
        map.get_mut(&tetra.p1).unwrap().push(tetra_index);
        map.get_mut(&tetra.p2).unwrap().push(tetra_index);
        map.get_mut(&tetra.p3).unwrap().push(tetra_index);
    }
    map
}
