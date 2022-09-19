use std::iter::Sum;

use glam::DVec2;

use super::dimension::Dimension;
use super::quantity::Quantity;

pub(super) const NONE: Dimension = Dimension {
    length: 0,
    time: 0,
    mass: 0,
};

impl<const D: Dimension> Quantity<f64, D> {
    pub fn abs(&self) -> Self {
        Self(self.0.abs())
    }

    pub fn zero() -> Self {
        Self(0.0)
    }

    pub fn max(&self, rhs: &Self) -> Self {
        Self(self.0.max(rhs.0))
    }

    pub fn squared(&self) -> Quantity<f64, { D.dimension_powi(2) }>
    where
        Quantity<f64, { D.dimension_powi(2) }>:,
    {
        Quantity::<f64, { D.dimension_powi(2) }>(self.0.powi(2))
    }

    pub fn cubed(&self) -> Quantity<f64, { D.dimension_powi(3) }>
    where
        Quantity<f64, { D.dimension_powi(3) }>:,
    {
        Quantity::<f64, { D.dimension_powi(3) }>(self.0.powi(3))
    }

    pub fn powi<const I: i32>(&self) -> Quantity<f64, { D.dimension_powi(I) }>
    where
        Quantity<f64, { D.dimension_powi(I) }>:,
    {
        Quantity::<f64, { D.dimension_powi(I) }>(self.0.powi(I))
    }
}

impl<const D: Dimension> Quantity<glam::DVec2, D> {
    pub fn new(x: Quantity<f64, D>, y: Quantity<f64, D>) -> Self {
        Self(DVec2::new(x.unwrap_value(), y.unwrap_value()))
    }

    pub fn abs(&self) -> Self {
        Self(self.0.abs())
    }

    pub fn zero() -> Self {
        Self(DVec2::new(0.0, 0.0))
    }

    pub fn x(&self) -> Quantity<f64, D> {
        Quantity(self.0.x)
    }

    pub fn y(&self) -> Quantity<f64, D> {
        Quantity(self.0.y)
    }

    pub fn length(&self) -> Quantity<f64, D> {
        Quantity::<f64, D>(self.0.length())
    }

    pub fn distance(&self, other: &Self) -> Quantity<f64, D> {
        Quantity::<f64, D>(self.0.distance(other.0))
    }

    pub fn distance_squared(&self, other: &Self) -> Quantity<f64, { D.dimension_powi(2) }>
    where
        Quantity<f64, { D.dimension_powi(2) }>:,
    {
        Quantity::<f64, { D.dimension_powi(2) }>(self.0.distance_squared(other.0))
    }

    pub fn normalize(&self) -> Self {
        Self(self.0.normalize())
    }
}

impl<const D: Dimension> Sum for Quantity<glam::DVec2, D> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut total = Self::zero();
        for item in iter {
            total += item;
        }
        total
    }
}

impl<const D: Dimension> Sum for Quantity<f64, D> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut total = Self::zero();
        for item in iter {
            total += item;
        }
        total
    }
}

macro_rules! unit_functions {
    ($($const: ident, $quantity:ident, $($dimension_name: ident: $dimension: literal),*, {$($unit:ident, $factor:literal, $($unit_symbol:literal)?),*}),+) => {
        use paste::paste;
        pub const UNIT_NAMES: &[(Dimension, &str, f64)] = &[
        $(
            $(
                $(
                    ($const, $unit_symbol, $factor),
                )*
            )*
        )*
        ];
        $(
            pub const $const: Dimension = Dimension {
                $(
                    $dimension_name: $dimension,
                )*
                .. NONE };
            pub type $quantity = Quantity<f64, $const>;
            paste!{
                pub type [<Vec $quantity>] = Quantity<glam::DVec2, $const>;
                pub type [<DVec2 $quantity>] = Quantity<glam::DVec2, $const>;
            }
            impl $quantity {
                $(
                    pub const fn $unit(v: f64) -> $quantity {
                        Quantity::<f64, $const>(v * $factor)
                    }
                )*
            }
            paste! {
            impl [<Vec $quantity>] {
                $(
                    pub fn $unit(x: f64, y: f64) -> Quantity::<glam::DVec2, $const> {
                        Quantity::<glam::DVec2, $const>(glam::DVec2::new(x, y) * $factor)
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
        meters, 1.0, "m",
        kilometers, 1000.0, "km",
        astronomical_units, 1.495978707e11, "au"
    },
    TIME, Time, time: 1,
    {
        seconds, 1.0, "s",
        years, 31557600.0, "yr"
    },
    VELOCITY, Velocity, length: 1, time: -1,
    {
        meters_per_second, 1.0, "m/s",
        astronomical_units_per_day, 1731460.0, "au/d"
    },
    MASS, Mass, mass: 1,
    {
        kilograms, 1.0, "kg",
        earth, 5.9722e24, "Mearth",
        solar, 1.988477e30, "Msol"
    },
    ACCELERATION, Acceleration, length: 1, time: -2,
    {
        meters_per_second_squared, 1.0, "m/s^2"
    },
    FORCE, Force, mass: 1, length: 1, time: -2,
    {
        newtons, 1.0, "N"
    },
    ENERGY, Energy, mass: 1, length: 2, time: -2,
    {
        joules, 1.0, "J"
    },
    DENSITY, Density, mass: 1, length: -2, time: 0,
    {
        kilogram_per_square_meter, 1.0, "kg/m^2"
    },
    PRESSURE, Pressure, mass: 1, length: -1, time: -2,
    {
        pascals, 1.0, "Pa"
    },
    LENGTHMASS, LengthMass, mass: 1, length: 1,
    {
    }
    );
