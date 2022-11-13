use crate::units::VecLength;
use crate::units::VecVelocity;

pub trait VelocityProfile: VelocityProfileClone {
    fn velocity(&self, pos: VecLength) -> VecVelocity;
}

#[derive(Clone)]
pub struct ConstantVelocity(pub VecVelocity);

impl VelocityProfile for ConstantVelocity {
    fn velocity(&self, _pos: VecLength) -> VecVelocity {
        self.0
    }
}

#[derive(Clone)]
pub struct ZeroVelocity;

impl VelocityProfile for ZeroVelocity {
    fn velocity(&self, _pos: VecLength) -> VecVelocity {
        VecVelocity::zero()
    }
}

pub trait VelocityProfileClone {
    fn clone_box(&self) -> Box<dyn VelocityProfile>;
}

impl<T> VelocityProfileClone for T
where
    T: 'static + VelocityProfile + Clone,
{
    fn clone_box(&self) -> Box<dyn VelocityProfile> {
        Box::new(self.clone())
    }
}
