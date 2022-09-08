use super::ParticleData;
use crate::domain::quadtree::NodeDataType;
use crate::units::Mass;
use crate::units::VecLength;
use crate::units::VecLengthMass;

#[derive(Default, Debug)]
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
    fn add_new_leaf_data(&mut self, pos: &VecLength, data: &ParticleData) {
        self.add_mass_at(pos, &data.mass);
    }
}
