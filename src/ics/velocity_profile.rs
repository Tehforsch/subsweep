use crate::units::VecLength;
use crate::units::VecVelocity;

pub trait VelocityProfile {
    fn velocity(&self, pos: VecLength) -> VecVelocity;
}

pub struct ConstantVelocity(pub VecVelocity);

impl VelocityProfile for ConstantVelocity {
    fn velocity(&self, _pos: VecLength) -> VecVelocity {
        self.0
    }
}

pub struct ZeroVelocity;

impl VelocityProfile for ZeroVelocity {
    fn velocity(&self, _pos: VecLength) -> VecVelocity {
        VecVelocity::zero()
    }
}
