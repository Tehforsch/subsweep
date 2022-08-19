use std::iter::Sum;

use glam::Vec2;

use super::dimension::Dimension;
use super::quantity::Quantity;

pub(super) const NONE: Dimension = Dimension {
    length: 0,
    time: 0,
    mass: 0,
};

impl<const D: Dimension> Quantity<f32, D> {
    pub fn abs(&self) -> Self {
        Self(self.0.abs())
    }

    pub fn zero() -> Self {
        Self(0.0)
    }

    pub fn squared(&self) -> Quantity<f32, { D.dimension_powi(2) }>
    where
        Quantity<f32, { D.dimension_powi(2) }>:,
    {
        Quantity::<f32, { D.dimension_powi(2) }>(self.0.powi(2))
    }

    pub fn cubed(&self) -> Quantity<f32, { D.dimension_powi(3) }>
    where
        Quantity<f32, { D.dimension_powi(3) }>:,
    {
        Quantity::<f32, { D.dimension_powi(3) }>(self.0.powi(3))
    }
}

impl<const D: Dimension> Quantity<glam::Vec2, D> {
    pub fn new(x: Quantity<f32, D>, y: Quantity<f32, D>) -> Self {
        Self(Vec2::new(x.unwrap_value(), y.unwrap_value()))
    }

    pub fn zero() -> Self {
        Self(Vec2::new(0.0, 0.0))
    }

    pub fn x(&self) -> Quantity<f32, D> {
        Quantity(self.0.x)
    }

    pub fn y(&self) -> Quantity<f32, D> {
        Quantity(self.0.y)
    }

    pub fn length(&self) -> Quantity<f32, D> {
        Quantity::<f32, D>(self.0.length())
    }

    pub fn distance(&self, other: &Self) -> Quantity<f32, D> {
        Quantity::<f32, D>(self.0.distance(other.0))
    }

    pub fn distance_squared(&self, other: &Self) -> Quantity<f32, { D.dimension_powi(2) }>
    where
        Quantity<f32, { D.dimension_powi(2) }>:,
    {
        Quantity::<f32, { D.dimension_powi(2) }>(self.0.distance_squared(other.0))
    }

    pub fn normalize(&self) -> Self {
        Self(self.0.normalize())
    }
}

impl<const D: Dimension> Sum for Quantity<glam::Vec2, D> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut total = Self::zero();
        for item in iter {
            total += item;
        }
        total
    }
}

macro_rules! unit_functions {
    ($storage_type:ty, $($const: ident, $quantity:ident, $($dimension_name: ident: $dimension: literal),*, {$($unit:ident, $factor:literal, $($unit_symbol:literal)?),+}),+) => {
        use super::Dimension;
        use super::Quantity;
        use super::NONE;
        pub const UNIT_NAMES: &[(Dimension, &str, f32)] = &[
        $(
            $(
                $(
                    ($const, $unit_symbol, $factor),
                )*
            )*
        )*
        ];
        $(
            const $const: Dimension = Dimension {
                $(
                    $dimension_name: $dimension,
                )*
                .. NONE };
            pub type $quantity = Quantity<$storage_type, $const>;
            $(
            pub fn $unit(v: $storage_type) -> $quantity {
                Quantity::<$storage_type, $const>(v * $factor)
            }
            )*
        )*
    }
}

#[rustfmt::skip]
macro_rules! implement_storage_type {
    ($type:ty) => {
        unit_functions!($type,
                    DIMENSIONLESS, Dimensionless, length: 0,
                    {
                        dimensionless, 1.0, ""
                    },
                    LENGTH, Length, length: 1,
                    {
                        meter, 1.0, "m",
                        kilometer, 1000.0, "km"
                    },
                    TIME, Time, time: 1,
                    {
                        second, 1.0, "s"
                    },
                    VELOCITY, Velocity, length: 1, time: -1,
                    {
                        meters_per_second, 1.0,
                    },
                    MASS, Mass, mass: 1,
                    {
                        kilogram, 1.0, "kg"
                    },
                    ACCELERATION, Acceleration, length: 1, time: -2,
                    {
                        meters_per_second_squared, 1.0,
                    },
                    FORCE, Force, mass: 1, length: 1, time: -2,
                    {
                        newton, 1.0, "N"
                    }
                    );
    }
}

pub mod vec2 {
    use glam::Vec2;
    implement_storage_type!(Vec2);
}

pub mod f32 {
    implement_storage_type!(f32);
}
