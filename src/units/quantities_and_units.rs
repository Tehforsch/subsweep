use std::ops::Mul;

pub use self::f64::*;
use super::dimension::Dimension;
use super::quantity::Quantity;

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

type Dimensionless<S> = Quantity<S, NONE>;
fn dimensionless<S>(v: S) -> Dimensionless<S>
where
    S: Mul<f64, Output = S>,
{
    Quantity::<S, NONE>(v * 1.0)
}

type Length<S> = Quantity<S, LENGTH>;
fn meter<S>(v: S) -> Length<S>
where
    S: Mul<f64, Output = S>,
{
    Quantity::<S, LENGTH>(v * 1.0)
}
fn kilometer<S>(v: S) -> Length<S>
where
    S: Mul<f64, Output = S>,
{
    Quantity::<S, LENGTH>(v * 1e3)
}

type Time<S> = Quantity<S, TIME>;
fn second<S>(v: S) -> Time<S>
where
    S: Mul<f64, Output = S>,
{
    Quantity::<S, TIME>(v * 1.0)
}

type Velocity<S> = Quantity<S, VELOCITY>;
fn meters_per_second<S>(v: S) -> Velocity<S>
where
    S: Mul<f64, Output = S>,
{
    Quantity::<S, VELOCITY>(v * 1.0)
}

impl<const D: Dimension> Quantity<f64, D> {
    pub fn abs(&self) -> Self {
        Self(self.0.abs())
    }
}

pub mod f64 {
    pub type Dimensionless = super::Dimensionless<f64>;
    pub type Length = super::Length<f64>;
    pub type Time = super::Time<f64>;
    pub type Velocity = super::Velocity<f64>;

    pub fn dimensionless(v: f64) -> Dimensionless {
        super::dimensionless(v)
    }
    pub fn meter(v: f64) -> Length {
        super::meter(v)
    }
    pub fn kilometer(v: f64) -> Length {
        super::kilometer(v)
    }
    pub fn second(v: f64) -> Time {
        super::second(v)
    }
    pub fn meters_per_second(v: f64) -> Velocity {
        super::meters_per_second(v)
    }
}
