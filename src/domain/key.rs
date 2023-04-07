use std::fmt::Debug;

use crate::peano_hilbert::PeanoHilbertKey;
use crate::peano_hilbert::NUM_BITS_PER_DIMENSION;

pub trait Key: PartialOrd + Ord + Copy + Clone + Debug {
    const MIN_VALUE: Self;
    const MAX_VALUE: Self;
    const MAX_DEPTH: usize;

    fn middle(start: Self, end: Self) -> Self;
}

impl Key for PeanoHilbertKey {
    const MIN_VALUE: PeanoHilbertKey = PeanoHilbertKey(0);
    const MAX_VALUE: PeanoHilbertKey = PeanoHilbertKey(u64::MAX);

    const MAX_DEPTH: usize = NUM_BITS_PER_DIMENSION as usize;

    fn middle(start: Self, end: Self) -> Self {
        Self(start.0 / 2 + end.0 / 2)
    }
}
