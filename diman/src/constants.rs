use super::dimension::Dimension;
use super::quantity::Quantity;
use crate::dimension::NONE;

const GRAVITY_CONSTANT_DIMENSION: Dimension = Dimension {
    length: 3,
    time: -2,
    mass: -1,
    ..NONE
};
pub const GRAVITY_CONSTANT: Quantity<f64, GRAVITY_CONSTANT_DIMENSION> = Quantity(6.67430e-11);
