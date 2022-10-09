use diman::Dimension;
use diman::Quantity;

const GRAVITY_CONSTANT_DIMENSION: Dimension = Dimension {
    length: 3,
    time: -2,
    mass: -1,
    temperature: 0,
};

// Todo: Turn this into a macro at some point
pub const GRAVITY_CONSTANT: Quantity<f64, GRAVITY_CONSTANT_DIMENSION> =
    Quantity::new_unchecked(6.67430e-11);
