use diman::define_constant;

use super::NONE;
use crate::prelude::Float;
use crate::units::Dimension;
use crate::units::Quantity;

macro_rules! constant {
    ($constant_name: ident, $value_base: literal, $($dimension_ident: ident: $dimension_expr: literal),*) => {
        define_constant!(Quantity, Float, NONE, $constant_name, $value_base, $($dimension_ident: $dimension_expr),*);
    }
}

constant!(GRAVITY_CONSTANT, 6.67430e-11, length: 3, time: -2, mass: -1);
constant!(BOLTZMANN_CONSTANT, 1.380649e-23, temperature: -1, length: 2, time: -2, mass: 1);
constant!(PROTON_MASS, 1.67262192369e-27, mass: 1);
