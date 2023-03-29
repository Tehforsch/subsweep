use bevy::prelude::Entity;
use generational_arena::Index;
use mpi::traits::Equivalence;

use super::SearchData;
use crate::communication::EntityKey;
use crate::voronoi::constructor::halo_iteration::IndexedSearchResult;
use crate::voronoi::constructor::halo_iteration::SearchResult;
use crate::voronoi::delaunay::TetraIndex;
use crate::voronoi::primitives::Float;
use crate::voronoi::Point2d;
use crate::voronoi::Point3d;
use crate::voronoi::ThreeD;
use crate::voronoi::TwoD;

pub trait IntoEquivalenceType {
    type Equiv: Equivalence;

    fn to_equivalent(&self) -> Self::Equiv;
    fn from_equivalent(equiv: &Self::Equiv) -> Self;
}

#[derive(Equivalence, Clone, Copy, Debug)]
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

#[derive(Equivalence, Clone)]
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

#[derive(Equivalence, Clone)]
pub struct SearchDataThreeDSend {
    point_x: Float,
    point_y: Float,
    point_z: Float,
    radius: Float,
    tetra_index: TetraIndexSend,
}

impl IntoEquivalenceType for SearchData<ThreeD> {
    type Equiv = SearchDataThreeDSend;

    fn to_equivalent(&self) -> Self::Equiv {
        SearchDataThreeDSend {
            point_x: self.point.x,
            point_y: self.point.y,
            point_z: self.point.z,
            radius: self.radius,
            tetra_index: self.tetra_index.into(),
        }
    }

    fn from_equivalent(equiv: &Self::Equiv) -> Self {
        SearchData::<ThreeD> {
            point: Point3d::new(equiv.point_x, equiv.point_y, equiv.point_z),
            radius: equiv.radius,
            tetra_index: equiv.tetra_index.into(),
        }
    }
}

#[derive(Equivalence, Clone, Debug)]
pub struct IndexedSearchResultThreeDSend {
    point_x: Float,
    point_y: Float,
    point_z: Float,
    tetra_index: TetraIndexSend,
    entity: EntityKey,
}

impl IntoEquivalenceType for IndexedSearchResult<ThreeD, Entity> {
    type Equiv = IndexedSearchResultThreeDSend;

    fn to_equivalent(&self) -> Self::Equiv {
        IndexedSearchResultThreeDSend {
            point_x: self.result.point.x,
            point_y: self.result.point.y,
            point_z: self.result.point.z,
            tetra_index: self.result.tetra_index.into(),
            entity: self.point_index.to_bits(),
        }
    }

    fn from_equivalent(equiv: &Self::Equiv) -> Self {
        IndexedSearchResult::<ThreeD, Entity> {
            result: SearchResult {
                point: Point3d::new(equiv.point_x, equiv.point_y, equiv.point_z),
                tetra_index: equiv.tetra_index.into(),
            },
            point_index: Entity::from_bits(equiv.entity),
        }
    }
}
