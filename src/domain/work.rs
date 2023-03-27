use std::cmp::Ordering;
use std::ops::Div;

use derive_more::Add;
use derive_more::AddAssign;
use derive_more::Mul;
use derive_more::Sub;
use derive_more::Sum;
use mpi::traits::Equivalence;

#[derive(Equivalence, Debug, Default, Clone, Copy, AddAssign, Add, Sub, Sum, Mul)]
pub struct Work(pub f64);

impl Div<Work> for Work {
    type Output = Work;

    fn div(self, rhs: Work) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Div<f64> for Work {
    type Output = Work;

    fn div(self, rhs: f64) -> Self::Output {
        Self(self.0 / rhs)
    }
}

// The following impls are taken from OrderedFloat. I can't wrap OrderedFloat directly because it doesnt implement
// equivalence
impl PartialOrd for Work {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Work {
    fn cmp(&self, other: &Self) -> Ordering {
        let lhs = &self.0;
        let rhs = &other.0;
        match lhs.partial_cmp(rhs) {
            Some(ordering) => ordering,
            None => {
                if lhs.is_nan() {
                    if rhs.is_nan() {
                        Ordering::Equal
                    } else {
                        Ordering::Greater
                    }
                } else {
                    Ordering::Less
                }
            }
        }
    }
}

impl PartialEq for Work {
    #[inline]
    fn eq(&self, other: &Work) -> bool {
        if self.0.is_nan() {
            other.0.is_nan()
        } else {
            self.0 == other.0
        }
    }
}

impl Eq for Work {}
