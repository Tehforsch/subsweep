use std::ops::Mul;

pub use self::f32::*;
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
    S: Mul<f32, Output = S>,
{
    Quantity::<S, NONE>(v * 1.0)
}

type Length<S> = Quantity<S, LENGTH>;
fn meter<S>(v: S) -> Length<S>
where
    S: Mul<f32, Output = S>,
{
    Quantity::<S, LENGTH>(v * 1.0)
}
fn kilometer<S>(v: S) -> Length<S>
where
    S: Mul<f32, Output = S>,
{
    Quantity::<S, LENGTH>(v * 1e3)
}

type Time<S> = Quantity<S, TIME>;
fn second<S>(v: S) -> Time<S>
where
    S: Mul<f32, Output = S>,
{
    Quantity::<S, TIME>(v * 1.0)
}

type Velocity<S> = Quantity<S, VELOCITY>;
fn meters_per_second<S>(v: S) -> Velocity<S>
where
    S: Mul<f32, Output = S>,
{
    Quantity::<S, VELOCITY>(v * 1.0)
}

impl<const D: Dimension> Quantity<f32, D> {
    pub fn abs(&self) -> Self {
        Self(self.0.abs())
    }
}

pub mod f32 {
    pub type Dimensionless = super::Dimensionless<f32>;
    pub type Length = super::Length<f32>;
    pub type Time = super::Time<f32>;
    pub type Velocity = super::Velocity<f32>;

    pub fn dimensionless(v: f32) -> Dimensionless {
        super::dimensionless(v)
    }
    pub fn meter(v: f32) -> Length {
        super::meter(v)
    }
    pub fn kilometer(v: f32) -> Length {
        super::kilometer(v)
    }
    pub fn second(v: f32) -> Time {
        super::second(v)
    }
    pub fn meters_per_second(v: f32) -> Velocity {
        super::meters_per_second(v)
    }
}

pub mod vec2 {
    use glam::Vec2;

    pub type Dimensionless = super::Dimensionless<Vec2>;
    pub type Length = super::Length<Vec2>;
    pub type Time = super::Time<Vec2>;
    pub type Velocity = super::Velocity<Vec2>;

    pub fn dimensionless(v: Vec2) -> Dimensionless {
        super::dimensionless(v)
    }
    pub fn meter(v: Vec2) -> Length {
        super::meter(v)
    }
    pub fn kilometer(v: Vec2) -> Length {
        super::kilometer(v)
    }
    pub fn second(v: Vec2) -> Time {
        super::second(v)
    }
    pub fn meters_per_second(v: Vec2) -> Velocity {
        super::meters_per_second(v)
    }
}
