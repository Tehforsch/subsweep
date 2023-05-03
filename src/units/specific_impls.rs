use super::Amount;
use super::Density;
use super::Dimension;
use super::Dimensionless;
use super::EnergyDensity;
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

    pub fn from_internal_energy_density_hydrogen_only(
        internal_energy_density: EnergyDensity,
        ionized_hydrogen_fraction: Dimensionless,
        density: Density,
    ) -> Self {
        let molecular_weight = 1.0 / (ionized_hydrogen_fraction + 1.0);
        (internal_energy_density / density).to_temperature(molecular_weight)
    }
}

impl EnergyPerMass {
    pub fn to_temperature(&self, molecular_weight: Dimensionless) -> Temperature {
        *self / (BOLTZMANN_CONSTANT / PROTON_MASS) / (1.0 / (GAMMA - 1.0)) * molecular_weight
    }
}

impl EnergyDensity {
    pub fn from_temperature_hydrogen_only(
        temperature: Temperature,
        ionized_hydrogen_fraction: Dimensionless,
        density: Density,
    ) -> Self {
        let molecular_weight = 1.0 / (ionized_hydrogen_fraction + 1.0);
        let internal_energy = temperature.to_internal_energy(molecular_weight);
        internal_energy * density
    }
}

impl Dimensionless {
    pub fn to_amount(&self) -> Amount {
        *self * Amount::one_unchecked()
    }
}
