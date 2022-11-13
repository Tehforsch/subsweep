use crate::units::Density;
use crate::units::VecLength;

pub trait DensityProfile: DensityProfileClone {
    fn density(&self, pos: VecLength) -> Density;
    fn max_value(&self) -> Density;
}

#[derive(Clone)]
pub struct ConstantDensity(pub Density);

impl DensityProfile for ConstantDensity {
    fn density(&self, _pos: VecLength) -> Density {
        self.0
    }

    fn max_value(&self) -> Density {
        self.0
    }
}

pub trait DensityProfileClone {
    fn clone_box(&self) -> Box<dyn DensityProfile>;
}

impl<T> DensityProfileClone for T
where
    T: 'static + DensityProfile + Clone,
{
    fn clone_box(&self) -> Box<dyn DensityProfile> {
        Box::new(self.clone())
    }
}
