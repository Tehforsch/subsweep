mod delaunay;
mod face;
mod indexed_arena;
mod tetra;

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
#[derive(Debug, Clone, Copy, From, Into, PartialEq, Eq)]
pub struct PointIndex(Index);

#[cfg(feature = "2d")]
pub type Point = glam::DVec2;
#[cfg(feature = "3d")]
pub type Point = glam::DVec3;

type TetraList = IndexedArena<TetraIndex, Tetra>;
type FaceList = IndexedArena<FaceIndex, Face>;
type PointList = IndexedArena<PointIndex, Point>;
