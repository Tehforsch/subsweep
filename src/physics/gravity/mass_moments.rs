use std::iter::Sum;
use std::ops::AddAssign;

use mpi::traits::Equivalence;

use crate::units::Mass;
use crate::units::VecLength;
use crate::units::VecLengthMass;

#[derive(Clone, Default, Equivalence)]
pub struct MassMoments {
    total: Mass,
    weighted_position_sum: VecLengthMass,
    count: usize,
}

impl MassMoments {
    pub fn total(&self) -> Mass {
        self.total
    }

    pub fn center_of_mass(&self) -> VecLength {
        if self.count == 0 {
            return VecLength::zero();
        }
        self.weighted_position_sum / self.total
    }

    pub fn add_mass_at(&mut self, pos: &VecLength, mass: &Mass) {
        self.count += 1;
        self.total += *mass;
        self.weighted_position_sum += *pos * *mass;
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

impl Sum<(Mass, VecLength)> for MassMoments {
    fn sum<I: Iterator<Item = (Mass, VecLength)>>(iter: I) -> Self {
        let mut s = Self::default();
        for (mass, pos) in iter {
            s.add_mass_at(&pos, &mass);
        }
        s
    }
}

impl AddAssign<&MassMoments> for MassMoments {
    fn add_assign(&mut self, rhs: &MassMoments) {
        self.count += rhs.count;
        self.total += rhs.total;
        self.weighted_position_sum += rhs.weighted_position_sum;
    }
}

impl std::fmt::Debug for MassMoments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Moments({:.3?} @ {:.3?})",
            self.total(),
            self.center_of_mass()
        )
    }
}
