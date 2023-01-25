mod constants;
mod dimension;
pub(crate) mod helpers;
mod specific_impls;

pub use constants::*;
use diman::define_system;
use diman::unit_system;
pub use dimension::Dimension;
pub use dimension::NONE;

define_system!(Quantity, Dimension, NONE, UNIT_NAMES);

#[rustfmt::skip]
unit_system!(
    Dimension,
    Quantity,
    NONE,
    UNIT_NAMES,
    DIMENSIONLESS, Dimensionless, length: 0,
    {
        dimensionless, 1.0, "dimensionless",
        percent, 0.01, "%"
    },
    LENGTH, Length, length: 1,
    {
        meters, 1.0, "m",
        centimeters, 1e-2, "cm",
        kilometers, 1000.0, "km",
        astronomical_units, 1.4959787e11, "au",
        parsec, 3.0857e16, "pc",
        kiloparsec, 3.0857e19, "kpc"
    },
    TIME, Time, time: 1,
    {
        seconds, 1.0, "s",
        years, 3.15576e7, "yr",
        kiloyears, 3.15576e10, "kyr",
        megayears, 3.15576e13, "Myr"
    },
    VELOCITY, Velocity, length: 1, time: -1,
    {
        meters_per_second, 1.0, "m/s",
        kilometers_per_second, 1000.0, "km/s",
        astronomical_units_per_day, 1731460.0, "au/d"
    },
    MASS, Mass, mass: 1,
    {
        kilograms, 1.0, "kg",
        grams, 1e-3, "g",
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
    ENERGYPERMASS, EnergyPerMass, mass: 0, length: 2, time: -2,
    {
        joules_per_kilogram, 1.0, "J/kg"
    },
    AREA, Area, length: 2,
    {
    },
    VOLUME2D, Volume2D, length: 2,
    {
        square_meters, 1.0, "m^2"
    },
    VOLUME3D, Volume3D, length: 3,
    {
        cubic_meters, 1.0, "m^3"
    },
    TEMPERATURE, Temperature, temperature: 1, 
    {
        kelvins, 1.0, "K"
    },
    PRESSURE3D, Pressure3D, mass: 1, length: -1, time: -2,
    {
        pascals, 1.0, "Pa"
    },
    PRESSURE2D, Pressure2D, mass: 1, length: 0, time: -2,
    {
        newtons_per_square_meter, 1.0, "N/m^2"
    },
    ENTROPY, Entropy, mass: 1, length: 2, time: -2, temperature: -1,
    {
    },
    DENSITY2D, Density2D, mass: 1, length: -2, time: 0,
    {
        kilogram_per_square_meter, 1.0, "kg/m^2"
    },
    DENSITY3D, Density3D, mass: 1, length: -3, time: 0,
    {
        kilogram_per_cubic_meter, 1.0, "kg/m^3"
    },
    NUMBERDENSITY3D, NumberDensity3D, length: -3,
    {
    },
    NUMBERDENSITY2D, NumberDensity2D, length: -2,
    {
    },
    AMOUNT, Amount, amount: 1,
    {
        photons, 1.0, "photons"
    },
    PHOTON_FLUX, PhotonFlux, time: -1, amount: 1,
    {
    },
    CROSS_SECTION_2D, CrossSection2D, length: 1,
    {
    },
    CROSS_SECTION_3D, CrossSection3D, length: 2,
    {
    },
    SOURCE_RATE, SourceRate, time: -1, amount: 1,
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

#[cfg(feature = "2d")]
mod reexport {
    pub type Density = super::Density2D;
    pub type NumberDensity = super::NumberDensity2D;
    pub type Pressure = super::Pressure2D;
    pub type Volume = super::Volume2D;
    pub type CrossSection = super::CrossSection2D;
}

#[cfg(not(feature = "2d"))]
mod reexport {
    pub type Density = super::Density3D;
    pub type NumberDensity = super::NumberDensity3D;
    pub type Pressure = super::Pressure3D;
    pub type Volume = super::Volume3D;
    pub type CrossSection = super::CrossSection3D;
}

pub use reexport::*;
