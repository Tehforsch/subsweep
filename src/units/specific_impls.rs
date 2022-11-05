use super::Dimension;
use super::Dimensionless;
use super::EnergyPerMass;
use super::Quantity;
use super::Temperature;
use super::VecLength;
use super::BOLTZMANN_CONSTANT;
use super::GAMMA;
use super::PROTON_MASS;
use crate::parameters::BoxSize;
use crate::prelude::Float;

impl<const D: Dimension> Quantity<Float, D> {
    pub fn one_unchecked() -> Self {
        Self(1.0)
    }
}

fn periodic_wrap_component(v: Float, min: Float, max: Float) -> Float {
    (v - min).rem_euclid(max - min)
}

impl VecLength {
    pub fn periodic_wrap(&mut self, box_size: &BoxSize) {
        self.0.x = periodic_wrap_component(
            self.0.x,
            box_size.min.x().value_unchecked(),
            box_size.max.x().value_unchecked(),
        );
        self.0.y = periodic_wrap_component(
            self.0.y,
            box_size.min.y().value_unchecked(),
            box_size.max.y().value_unchecked(),
        );
        #[cfg(not(feature = "2d"))]
        {
            self.0.z = periodic_wrap_component(
                self.0.z,
                box_size.min.z().value_unchecked(),
                box_size.max.z().value_unchecked(),
            );
        }
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
