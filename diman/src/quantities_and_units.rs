use super::quantity::Quantity;
use crate::dimension::NONE;
use crate::Dimension;

macro_rules! unit_system {
    ($dimension: ident, $quantity: ident, $($const: ident, $quantity_name:ident, $($dimension_name: ident: $dimension_value: literal),*, {$($unit:ident, $factor:literal, $($unit_symbol:literal)?),*}),+) => {
        use paste::paste;
        pub const UNIT_NAMES: &[($dimension, &str, f64)] = &[
        $(
            $(
                $(
                    ($const, $unit_symbol, $factor),
                )*
            )*
        )*
        ];
        $(
            pub const $const: $dimension = $dimension {
                $(
                    $dimension_name: $dimension_value,
                )*
                .. NONE };
            pub type $quantity_name = $quantity<f64, $const>;
            paste!{
                pub type [<Vec $quantity_name>] = $quantity<glam::DVec2, $const>;
                pub type [<DVec2 $quantity_name>] = $quantity<glam::DVec2, $const>;
            }
            impl $quantity_name {
                $(
                    pub const fn $unit(v: f64) -> $quantity_name {
                        $quantity::<f64, $const>(v * $factor)
                    }
                )*
            }
            paste! {
            impl [<Vec $quantity_name>] {
                $(
                    pub fn $unit(x: f64, y: f64) -> $quantity::<glam::DVec2, $const> {
                        $quantity::<glam::DVec2, $const>(glam::DVec2::new(x, y) * $factor)
                    }
                )*
            }
            }
        )*
    }
}

#[rustfmt::skip]
unit_system!(
    Dimension,
    Quantity,
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
    VOLUME, Volume, mass: 0, length: 3, time: 0,
    {
    },
    PRESSURE, Pressure, mass: 1, length: -1, time: -2,
    {
        pascals, 1.0, "Pa"
    },
    ENTROPY, Entropy, mass: 1, length: 2, time: -2, temperature: -1,
    {
    },
    ENTROPIC_FUNCTION, EntropicFunction, length: 4, mass: -1, time: 2,
    {
    },
    NUMBERDENSITY3D, NumberDensity3D, length: -3,
    {
    },
    NUMBERDENSITY2D, NumberDensity2D, length: -2,
    {
    },
    LENGTHMASS, LengthMass, mass: 1, length: 1,
    {
    },
    INVERSE_TIME, InverseTime, time: -1,
    {
    },
    INVERSE_TIME_SQUARED, InverseTimeSquared, time: -2,
    {
    }
    );
