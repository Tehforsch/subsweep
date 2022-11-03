use crate::units::Density;
use crate::units::VecLength;

pub trait DensityProfile {
    fn density(&self, pos: VecLength) -> Density;
    fn max_value(&self) -> Density;
}

pub struct ConstantDensity(pub Density);

impl DensityProfile for ConstantDensity {
    fn density(&self, _pos: VecLength) -> Density {
        self.0
    }

    fn max_value(&self) -> Density {
        self.0
    }
}
