use generational_arena::Index;
use mpi::traits::Equivalence;

use super::SearchData;
use crate::voronoi::delaunay::TetraIndex;
use crate::voronoi::primitives::Float;
use crate::voronoi::Point2d;
use crate::voronoi::TwoD;

pub trait IntoEquivalenceType {
    type Equiv: Equivalence;

    fn to_equivalent(&self) -> Self::Equiv;
    fn from_equivalent(equiv: &Self::Equiv) -> Self;
}

#[derive(Equivalence, Clone, Copy)]
pub struct TetraIndexSend {
    gen: usize,
    index: u64,
}

impl From<TetraIndex> for TetraIndexSend {
    fn from(value: TetraIndex) -> Self {
        let (gen, index) = value.0.into_raw_parts();
        Self { gen, index }
    }
}

impl From<TetraIndexSend> for TetraIndex {
    fn from(value: TetraIndexSend) -> Self {
        TetraIndex(Index::from_raw_parts(value.gen, value.index))
    }
}

#[derive(Equivalence)]
pub struct SearchDataTwoDSend {
    point_x: Float,
    point_y: Float,
    radius: Float,
    tetra_index: TetraIndexSend,
}

impl IntoEquivalenceType for SearchData<TwoD> {
    type Equiv = SearchDataTwoDSend;

    fn to_equivalent(&self) -> Self::Equiv {
        SearchDataTwoDSend {
            point_x: self.point.x,
            point_y: self.point.y,
            radius: self.radius,
            tetra_index: self.tetra_index.into(),
        }
    }

    fn from_equivalent(equiv: &Self::Equiv) -> Self {
        SearchData::<TwoD> {
            point: Point2d::new(equiv.point_x, equiv.point_y),
            radius: equiv.radius,
            tetra_index: equiv.tetra_index.into(),
        }
    }
}
