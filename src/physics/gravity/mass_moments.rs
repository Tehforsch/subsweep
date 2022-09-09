use std::iter::Sum;

use mpi::traits::Equivalence;

use super::ParticleData;
use crate::domain::quadtree::NodeDataType;
use crate::units::Mass;
use crate::units::Vec2Length;
use crate::units::VecLength;
use crate::units::VecLengthMass;

#[derive(Equivalence, Default, Clone)]
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
}

impl NodeDataType<VecLength, ParticleData> for MassMoments {
    fn handle_insertion(&mut self, pos: &VecLength, data: &ParticleData) {
        self.add_mass_at(pos, &data.mass);
    }
}

impl Sum<(Mass, Vec2Length)> for MassMoments {
    fn sum<I: Iterator<Item = (Mass, Vec2Length)>>(iter: I) -> Self {
        let mut s = Self::default();
        for (mass, pos) in iter {
            s.add_mass_at(&pos, &mass);
        }
        s
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
