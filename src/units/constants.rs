use super::dimension::Dimension;
use super::quantity::Quantity;

const GRAVITY_CONSTANT_DIMENSION: Dimension = Dimension {
    length: 3,
    time: -2,
    mass: -1,
};
pub const GRAVITY_CONSTANT: Quantity<f32, GRAVITY_CONSTANT_DIMENSION> = Quantity(6.67430e-11);
