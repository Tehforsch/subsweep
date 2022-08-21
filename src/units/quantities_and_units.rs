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

    pub fn max(&self, rhs: &Self) -> Self {
        Self(self.0.max(rhs.0))
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
    ($($const: ident, $quantity:ident, $($dimension_name: ident: $dimension: literal),*, {$($unit:ident, $factor:literal, $($unit_symbol:literal)?),+}),+) => {
        use paste::paste;
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
            pub type $quantity = Quantity<f32, $const>;
            paste!{
                pub type [<Vec $quantity>] = Quantity<glam::Vec2, $const>;
                pub type [<Vec2 $quantity>] = Quantity<glam::Vec2, $const>;
            }
            impl $quantity {
                $(
                    pub fn $unit(v: f32) -> $quantity {
                        Quantity::<f32, $const>(v * $factor)
                    }
                )*
            }
            paste! {
            impl [<Vec $quantity>] {
                $(
                    pub fn $unit(x: f32, y: f32) -> Quantity::<glam::Vec2, $const> {
                        Quantity::<glam::Vec2, $const>(glam::Vec2::new(x, y) * $factor)
                    }
                )*
            }
            }
        )*
    }
}

#[rustfmt::skip]
unit_functions!(
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
