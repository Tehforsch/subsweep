use diman::define_constant;

use super::NONE;
use crate::prelude::Float;
use crate::units::Dimension;
use crate::units::Quantity;

#[macro_export]
macro_rules! constant {
    ($constant_name: ident, $value_base: expr, $($dimension_ident: ident: $dimension_expr: literal),*) => {
        define_constant!(Quantity, Float, NONE, $constant_name, $value_base, $($dimension_ident: $dimension_expr),*);
    }
}

constant!(GRAVITY_CONSTANT, 6.67430e-11, length: 3, time: -2, mass: -1);
constant!(BOLTZMANN_CONSTANT, 1.380649e-23, temperature: -1, length: 2, time: -2, mass: 1);
constant!(PROTON_MASS, 1.67262192369e-27, mass: 1);
constant!(SPEED_OF_LIGHT, 299792458.0, length: 1, time: -1);
pub const GAMMA: Float = 5.0 / 3.0;

#[cfg(not(feature = "2d"))]
constant!(SWEEP_HYDROGEN_ONLY_CROSS_SECTION, 5.339944e-22, length: 2);

// This is probably wrong
#[cfg(feature = "2d")]
constant!(SWEEP_HYDROGEN_ONLY_CROSS_SECTION, 5.339944e-22, length: 1);

#[cfg(not(feature = "2d"))]
constant!(CASE_B_RECOMBINATION_RATE_HYDROGEN, 2.59e-19, length: 3, time: -1);

#[cfg(feature = "2d")]
constant!(CASE_B_RECOMBINATION_RATE_HYDROGEN, 2.59e-19, length: 2, time: -1);
