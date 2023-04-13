use mpi::traits::Equivalence;

use super::SearchData;
use crate::dimension::ThreeD;
use crate::dimension::TwoD;
use crate::prelude::ParticleId;
use crate::simulation_box::PeriodicWrapType2d;
use crate::simulation_box::PeriodicWrapType3d;
use crate::simulation_box::WrapType;
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

#[derive(Clone, Debug, Equivalence)]
pub struct SearchResultTwoDSend {
    point_x: Float,
    point_y: Float,
    id: ParticleId,
    periodic_wrap_type: (isize, isize),
}

impl IntoEquivalenceType for SearchResult<TwoD> {
    type Equiv = SearchResultTwoDSend;

    fn to_equivalent(&self) -> Self::Equiv {
        SearchResultTwoDSend {
            point_x: self.point.x,
            point_y: self.point.y,
            id: self.id,
            periodic_wrap_type: self.periodic_wrap_type.into(),
        }
    }

    fn from_equivalent(equiv: &Self::Equiv) -> Self {
        SearchResult {
            point: Point2d::new(equiv.point_x, equiv.point_y),
            id: equiv.id,
            periodic_wrap_type: equiv.periodic_wrap_type.into(),
        }
    }
}

#[derive(Clone, Debug, Equivalence)]
pub struct SearchResultThreeDSend {
    point_x: Float,
    point_y: Float,
    point_z: Float,
    id: ParticleId,
    periodic_wrap_type: (isize, isize, isize),
}

impl IntoEquivalenceType for SearchResult<ThreeD> {
    type Equiv = SearchResultThreeDSend;

    fn to_equivalent(&self) -> Self::Equiv {
        SearchResultThreeDSend {
            point_x: self.point.x,
            point_y: self.point.y,
            point_z: self.point.z,
            id: self.id,
            periodic_wrap_type: self.periodic_wrap_type.into(),
        }
    }

    fn from_equivalent(equiv: &Self::Equiv) -> Self {
        SearchResult {
            point: Point3d::new(equiv.point_x, equiv.point_y, equiv.point_z),
            id: equiv.id,
            periodic_wrap_type: equiv.periodic_wrap_type.into(),
        }
    }
}

impl From<PeriodicWrapType2d> for (isize, isize) {
    fn from(value: PeriodicWrapType2d) -> Self {
        (value.x.into(), value.y.into())
    }
}

impl From<(isize, isize)> for PeriodicWrapType2d {
    fn from((x, y): (isize, isize)) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }
}

impl From<PeriodicWrapType3d> for (isize, isize, isize) {
    fn from(value: PeriodicWrapType3d) -> Self {
        (value.x.into(), value.y.into(), value.z.into())
    }
}

impl From<(isize, isize, isize)> for PeriodicWrapType3d {
    fn from((x, y, z): (isize, isize, isize)) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        }
    }
}

impl From<WrapType> for isize {
    fn from(value: WrapType) -> Self {
        use WrapType::*;
        match value {
            NoWrap => 0,
            Minus => -1,
            Plus => 1,
        }
    }
}

impl From<isize> for WrapType {
    fn from(value: isize) -> Self {
        match value {
            -1 => WrapType::Minus,
            0 => WrapType::NoWrap,
            1 => WrapType::Plus,
            _ => panic!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::simulation_box::WrapType;

    #[test]
    fn conversion_symmetric() {
        let check_symmetric = |x: WrapType| {
            let v1: isize = x.into();
            let v2: WrapType = v1.into();
            assert_eq!(v2, x);
        };
        check_symmetric(WrapType::NoWrap);
        check_symmetric(WrapType::Plus);
        check_symmetric(WrapType::Minus);
    }
}
