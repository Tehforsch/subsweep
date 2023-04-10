use mpi::traits::Equivalence;

use super::SearchData;
use crate::dimension::ThreeD;
use crate::dimension::TwoD;
use crate::prelude::ParticleId;
use crate::voronoi::constructor::halo_iteration::SearchResult;
use crate::voronoi::primitives::Float;
use crate::voronoi::Point2d;
use crate::voronoi::Point3d;

pub trait IntoEquivalenceType {
    type Equiv: Equivalence;

    fn to_equivalent(&self) -> Self::Equiv;
    fn from_equivalent(equiv: &Self::Equiv) -> Self;
}

#[derive(Equivalence, Clone)]
pub struct SearchDataTwoDSend {
    point_x: Float,
    point_y: Float,
    radius: Float,
}

impl IntoEquivalenceType for SearchData<TwoD> {
    type Equiv = SearchDataTwoDSend;

    fn to_equivalent(&self) -> Self::Equiv {
        SearchDataTwoDSend {
            point_x: self.point.x,
            point_y: self.point.y,
            radius: self.radius,
        }
    }

    fn from_equivalent(equiv: &Self::Equiv) -> Self {
        SearchData::<TwoD> {
            point: Point2d::new(equiv.point_x, equiv.point_y),
            radius: equiv.radius,
        }
    }
}

#[derive(Equivalence, Clone)]
pub struct SearchDataThreeDSend {
    point_x: Float,
    point_y: Float,
    point_z: Float,
    radius: Float,
}

impl IntoEquivalenceType for SearchData<ThreeD> {
    type Equiv = SearchDataThreeDSend;

    fn to_equivalent(&self) -> Self::Equiv {
        SearchDataThreeDSend {
            point_x: self.point.x,
            point_y: self.point.y,
            point_z: self.point.z,
            radius: self.radius,
        }
    }

    fn from_equivalent(equiv: &Self::Equiv) -> Self {
        SearchData::<ThreeD> {
            point: Point3d::new(equiv.point_x, equiv.point_y, equiv.point_z),
            radius: equiv.radius,
        }
    }
}

#[derive(Equivalence, Clone, Debug)]
pub struct SearchResultThreeDSend {
    point_x: Float,
    point_y: Float,
    point_z: Float,
    id: ParticleId,
}

impl IntoEquivalenceType for SearchResult<ThreeD> {
    type Equiv = SearchResultThreeDSend;

    fn to_equivalent(&self) -> Self::Equiv {
        SearchResultThreeDSend {
            point_x: self.point.x,
            point_y: self.point.y,
            point_z: self.point.z,
            id: self.id,
        }
    }

    fn from_equivalent(equiv: &Self::Equiv) -> Self {
        SearchResult {
            point: Point3d::new(equiv.point_x, equiv.point_y, equiv.point_z),
            id: equiv.id,
        }
    }
}
