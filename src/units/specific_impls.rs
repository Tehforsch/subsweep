use super::Density;
use super::Dimension;
use super::Dimensionless;
use super::EnergyDensity;
use super::EnergyPerMass;
use super::Length;
use super::Quantity;
use super::Temperature;
use super::BOLTZMANN_CONSTANT;
use super::GAMMA;
use super::PROTON_MASS;
use crate::parameters::Cosmology;
use crate::prelude::Float;

impl<const D: Dimension> Quantity<Float, D> {
    pub fn one_unchecked() -> Self {
        Self(1.0)
    }
}

impl<const D: Dimension, S> Quantity<S, D> {
    pub fn dimension(&self) -> Dimension {
        D
    }
}

#[cfg(feature = "3d")]
impl super::Vec3Length {
    pub fn from_vector_and_scale(m: super::MVec3, l: Length) -> super::Vec3Length {
        super::Vec3Length::new(m.x * l, m.y * l, m.z * l)
    }
}

#[cfg(feature = "2d")]
impl super::Vec2Length {
    pub fn from_vector_and_scale(m: super::MVec2, l: Length) -> super::Vec2Length {
        super::Vec2Length::new(m.x * l, m.y * l)
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
        *self * molecular_weight * (GAMMA - 1.0) * PROTON_MASS / BOLTZMANN_CONSTANT
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

impl<const D: Dimension, S> Quantity<S, D>
where
    Quantity<S, { Dimension::non_cosmological(D) }>:,
    S: std::ops::Mul<f64, Output = S>,
{
    pub fn make_non_cosmological(
        self,
        cosmology: &Cosmology,
    ) -> Quantity<S, { Dimension::non_cosmological(D) }> {
        Quantity::new_unchecked(self.0 * cosmology.get_factor(&D))
    }
}
