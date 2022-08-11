use super::dimension::Dimension;
use super::quantity::Quantity;

pub(super) const NONE: Dimension = Dimension {
    length: 0,
    time: 0,
    mass: 0,
};
pub type Dimensionless = Quantity<NONE>;
pub fn dimensionless(v: f64) -> Dimensionless {
    Quantity::<NONE>(1.0 * v)
}

const LENGTH: Dimension = Dimension { length: 1, ..NONE };
pub type Length = Quantity<LENGTH>;
pub fn meter(v: f64) -> Length {
    Quantity::<LENGTH>(1.0 * v)
}
pub fn kilometer(v: f64) -> Length {
    Quantity::<LENGTH>(1e3 * v)
}

const TIME: Dimension = Dimension { time: 1, ..NONE };
pub type Time = Quantity<TIME>;
pub fn second(v: f64) -> Time {
    Quantity::<TIME>(1.0 * v)
}
