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

macro_rules! quantity_definitions {
    ($storage_type:ty, $($name:ident, $dimension:ty, $factor:literal),+) => {
        $(
        pub fn $name(v: $storage_type) -> $dimension {
            super::$name(v)
        }
        )*
    }
}

macro_rules! unit_functions {
    ($storage_type:ty, $($name:ident, $dimension:ty, $factor:literal),+) => {
        $(
        pub fn $name(v: $storage_type) -> $dimension {
            super::$name(v)
        }
        )*
    }
}

macro_rules! implement_storage_type {
    ($type:ty) => {
        pub type Dimensionless = super::Dimensionless<$type>;
        pub type Length = super::Length<$type>;
        pub type Time = super::Time<$type>;
        pub type Velocity = super::Velocity<$type>;

        // quantities!(&type,
        //             Dimensionless, DIMENSIONLESS, { ..NONE },
        //             Length, LENGTH, { length: 1, ..NONE },
        //             Time, TIME, { time: 1, ..NONE },
        //             Velocity, VELOCITY, { length: 1, time: -1, ..NONE
        //             )

        unit_functions!(
            $type,
            dimensionless,
            Dimensionless,
            1.0,
            meter,
            Length,
            1.0,
            kilometer,
            Length,
            1000.0,
            second,
            Time,
            1.0,
            meters_per_second,
            Velocity,
            1.0
        );
    };
}

pub mod vec2 {
    use glam::Vec2;
    implement_storage_type!(Vec2);
}

pub mod f32 {
    implement_storage_type!(f32);
}
