use std::fmt::Debug;

use crate::peano_hilbert::PeanoKey2d;
use crate::peano_hilbert::PeanoKey3d;
use crate::peano_hilbert::NUM_BITS_PER_DIMENSION_2D;
use crate::peano_hilbert::NUM_BITS_PER_DIMENSION_3D;

pub trait Key: PartialOrd + Ord + Copy + Clone + Debug {
    const MIN_VALUE: Self;
    const MAX_VALUE: Self;
    const MAX_DEPTH: usize;

    fn middle(start: Self, end: Self) -> Self;
}

impl Key for PeanoKey2d {
    const MIN_VALUE: PeanoKey2d = PeanoKey2d(0);
    const MAX_VALUE: PeanoKey2d = PeanoKey2d(u64::MAX);

    const MAX_DEPTH: usize = NUM_BITS_PER_DIMENSION_2D as usize;

    fn middle(start: Self, end: Self) -> Self {
        Self(start.0 / 2 + end.0 / 2)
    }
}

impl Key for PeanoKey3d {
    const MIN_VALUE: PeanoKey3d = PeanoKey3d(0);
    const MAX_VALUE: PeanoKey3d = PeanoKey3d(u128::MAX);

    const MAX_DEPTH: usize = NUM_BITS_PER_DIMENSION_3D as usize;

    fn middle(start: Self, end: Self) -> Self {
        Self(start.0 / 2 + end.0 / 2)
    }
}
