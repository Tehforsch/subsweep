use crate::prelude::Float;
use crate::units::NumberDensity;
use crate::units::Volume;

#[derive(Clone)]
pub enum Resolution {
    NumberDensity(NumberDensity),
    NumParticles(usize),
}

impl Resolution {
    pub fn as_number_density(&self, volume: Volume) -> NumberDensity {
        match self {
            Self::NumberDensity(density) => *density,
            Self::NumParticles(num) => *num as Float / volume,
        }
    }
}
