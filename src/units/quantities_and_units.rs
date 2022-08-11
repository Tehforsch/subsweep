pub use self::f64::*;
use super::dimension::Dimension;

pub(super) const NONE: Dimension = Dimension {
    length: 0,
    time: 0,
    mass: 0,
};
const LENGTH: Dimension = Dimension { length: 1, ..NONE };
const TIME: Dimension = Dimension { time: 1, ..NONE };
const VELOCITY: Dimension = Dimension {
    length: 1,
    time: -1,
    ..NONE
};

mod f64 {
    use super::super::quantity::Quantity;
    use super::*;

    pub type Dimensionless = Quantity<f64, NONE>;
    pub fn dimensionless(v: f64) -> Dimensionless {
        Quantity::<f64, NONE>(1.0 * v)
    }

    pub type Length = Quantity<f64, LENGTH>;
    pub fn meter(v: f64) -> Length {
        Quantity::<f64, LENGTH>(1.0 * v)
    }
    pub fn kilometer(v: f64) -> Length {
        Quantity::<f64, LENGTH>(1e3 * v)
    }

    pub type Time = Quantity<f64, TIME>;
    pub fn second(v: f64) -> Time {
        Quantity::<f64, TIME>(1.0 * v)
    }

    pub type Velocity = Quantity<f64, VELOCITY>;
    pub fn meters_per_second(v: f64) -> Velocity {
        Quantity::<f64, VELOCITY>(1.0 * v)
    }

    impl<const D: Dimension> Quantity<f64, D> {
        pub fn abs(&self) -> Self {
            Self(self.0.abs())
        }
    }
}
