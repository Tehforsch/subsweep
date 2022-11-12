use super::Dimension;
use super::Dimensionless;
use super::EnergyPerMass;
use super::Quantity;
use super::Temperature;
use super::BOLTZMANN_CONSTANT;
use super::GAMMA;
use super::PROTON_MASS;
use crate::prelude::Float;

impl<const D: Dimension> Quantity<Float, D> {
    pub fn one_unchecked() -> Self {
        Self(1.0)
    }
}

impl Temperature {
    pub fn to_internal_energy(&self, molecular_weight: Dimensionless) -> EnergyPerMass {
        *self * (BOLTZMANN_CONSTANT / PROTON_MASS) * (1.0 / (GAMMA - 1.0)) / molecular_weight
    }
}

impl EnergyPerMass {
    pub fn to_temperature(&self, molecular_weight: Dimensionless) -> Temperature {
        *self / (BOLTZMANN_CONSTANT / PROTON_MASS) / (1.0 / (GAMMA - 1.0)) * molecular_weight
    }
}
