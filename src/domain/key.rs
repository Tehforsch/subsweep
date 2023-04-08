use std::fmt::Debug;

use crate::dimension::Dimension;
use crate::dimension::ThreeD;
use crate::dimension::TwoD;
use crate::extent::Extent;
use crate::peano_hilbert::PeanoKey2d;
use crate::peano_hilbert::PeanoKey3d;
use crate::peano_hilbert::NUM_BITS_PER_DIMENSION_2D;
use crate::peano_hilbert::NUM_BITS_PER_DIMENSION_3D;
use crate::units::MVec2;
use crate::units::MVec3;
use crate::units::Vec2Length;
use crate::units::Vec3Length;

pub trait Key: PartialOrd + Ord + Copy + Clone + Debug {
    type Dimension: Dimension;

    const MIN_VALUE: Self;
    const MAX_VALUE: Self;
    const MAX_DEPTH: usize;

    fn middle(start: Self, end: Self) -> Self;
}

impl Key for PeanoKey2d {
    type Dimension = TwoD;
    const MIN_VALUE: PeanoKey2d = PeanoKey2d(0);
    const MAX_VALUE: PeanoKey2d = PeanoKey2d(u64::MAX);

    const MAX_DEPTH: usize = NUM_BITS_PER_DIMENSION_2D as usize;

    fn middle(start: Self, end: Self) -> Self {
        Self(start.0 / 2 + end.0 / 2)
    }
}

impl Key for PeanoKey3d {
    type Dimension = ThreeD;
    const MIN_VALUE: PeanoKey3d = PeanoKey3d(0);
    const MAX_VALUE: PeanoKey3d = PeanoKey3d(u128::MAX);

    const MAX_DEPTH: usize = NUM_BITS_PER_DIMENSION_3D as usize;

    fn middle(start: Self, end: Self) -> Self {
        Self(start.0 / 2 + end.0 / 2)
    }
}

pub trait IntoKey: Sized {
    type Key: Key;
    fn into_key(self, extent: &Extent<Self>) -> Self::Key;
}

impl IntoKey for MVec2 {
    type Key = PeanoKey2d;

    fn into_key(self, extent: &Extent<Self>) -> Self::Key {
        PeanoKey2d::from_point_and_min_max(self, extent.min, extent.max)
    }
}

impl IntoKey for MVec3 {
    type Key = PeanoKey3d;

    fn into_key(self, extent: &Extent<Self>) -> Self::Key {
        PeanoKey3d::from_point_and_min_max(self, extent.min, extent.max)
    }
}

impl IntoKey for Vec2Length {
    type Key = PeanoKey2d;

    fn into_key(self, extent: &Extent<Self>) -> Self::Key {
        PeanoKey2d::from_point_and_min_max(
            self.value_unchecked(),
            extent.min.value_unchecked(),
            extent.max.value_unchecked(),
        )
    }
}

impl IntoKey for Vec3Length {
    type Key = PeanoKey3d;

    fn into_key(self, extent: &Extent<Self>) -> Self::Key {
        PeanoKey3d::from_point_and_min_max(
            self.value_unchecked(),
            extent.min.value_unchecked(),
            extent.max.value_unchecked(),
        )
    }
}
