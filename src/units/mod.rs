mod dimension;
pub(crate) mod helpers;
mod specific_impls;

use diman::unit_system;
pub use dimension::Dimension;
pub use dimension::NONE;

#[rustfmt::skip]
unit_system!(
    Quantity,
    Dimension,
    [
        def Dimensionless = {},
        unit dimensionless = Dimensionless,
        unit (percent, "%") = 0.01 * Dimensionless,
        def Length = { length: 1 },
        unit (meters, "m") = Length,
        unit (centimeters, "cm") = 0.01 * meters,
        unit (kilometers, "km") = 1000.0 * meters,
        unit (parsec, "pc") = 3.0857e16 * meters,
        unit (kiloparsec, "kpc") = 1000 * parsec,
        def Time = { time: 1 },
        unit (seconds, "s") = 1.0 * Time,
        unit (years, "yr") = 3.15576e7 * seconds,
        unit (kiloyears, "kyr") = 1000.0 * years,
        unit (megayears, "Myr") = 1e6 * years,
        def Mass = { mass: 1 },
        unit (kilograms, "kg") = Mass,
        unit (grams, "g") = 1e-3 * kilograms,
        unit (solar, "Msol") = 1.988477e30 * kilograms,
        def Velocity = Length / Time,
        unit (meters_per_second, "m/s") = meters / seconds,
        def Energy = Mass * Velocity * Velocity,
        unit (joules, "J") = 1.0 * Energy,
        unit (ergs, "J") = 1e-7 * joules,
        unit (electron_volts, "eV") = 1.602176634e-19 * joules,
        def Temperature = { temperature: 1 },
        def InverseTemperature = Dimensionless / Temperature,
        unit (kelvins, "K") = Temperature,
        def Area = Length * Length,
        unit square_meters = Area,
        unit (square_centimeters, "cm^2") = 1e-4 * square_meters,
        def Force = Energy / Length,
        def EnergyDensity = Energy / Volume3D,
        def EnergyPerMass = Energy / Mass,
        def EnergyPerTime = Energy / Time,
        unit ergs_per_s = ergs / seconds,
        def Volume2D = Length * Length,
        def Volume3D = Length * Length * Length,
        unit cubic_meters = Volume3D,
        unit cubic_centimeters = 1e-6 * cubic_meters,
        def Density = Mass / Volume3D,
        def Rate = Dimensionless / Time,
        def PhotonRate = Rate,
        def SourceRate = Rate,
        unit photons_per_second = 1.0 / seconds,
        def PhotonFlux = PhotonRate / Area,
        unit photons_per_s_per_cm_squared = photons_per_second / centimeters_squared,
        def CrossSection = Area,
        unit (centimeters_squared, "cm^2") = centimeters * centimeters,
        def VolumeRate = Volume3D / Time,
        unit (centimeters_cubed_per_s, "cm^3/s") = centimeters * centimeters * centimeters / seconds,
        def HeatingTerm = Energy * Volume3D / Time,
        unit (ergs_centimeters_cubed_per_s, "cm^3/s") = ergs * centimeters_cubed_per_s,
        def HeatingRate = Energy / (Volume3D * Time),

        def NumberDensity = Dimensionless / Volume3D,
        unit per_centimeters_cubed = 1.0 / cubic_centimeters,

        constant BOLTZMANN_CONSTANT = 1.380649e-23 * joules / kelvins,
        constant PROTON_MASS = 1.67262192369e-27 * kilograms,
        constant SPEED_OF_LIGHT = 299792458.0 * meters_per_second,
        constant GAMMA = 5.0 / 3.0,
        constant SWEEP_HYDROGEN_ONLY_CROSS_SECTION = 5.339944e-18 * square_centimeters,
        constant SWEEP_HYDROGEN_ONLY_AVERAGE_PHOTON_ENERGY = 1.210624e-12 * ergs,
        constant CASE_B_RECOMBINATION_RATE_HYDROGEN = 2.59e-19 * cubic_meters / seconds,
    ]
);

pub use self::f64::*;

#[cfg(feature = "2d")]
mod reexport {
    use super::Dimension;

    pub type Density = super::Density2D;
    pub type NumberDensity = super::NumberDensity2D;
    pub type Volume = super::Volume2D;
    pub type CrossSection = super::CrossSection2D;
    pub type MVec = super::MVec2;
}

#[cfg(not(feature = "2d"))]
mod reexport {
    pub type Volume = super::Volume3D;
    pub type VecLength = super::dvec3::Length;
    pub type VecDimensionless = super::dvec3::Dimensionless;
    pub type MVec = super::MVec3;
}

pub type MVec2 = glam::DVec2;
pub type MVec3 = glam::DVec3;
pub type Vec2Length = self::dvec2::Length;
pub type Vec3Length = self::dvec3::Length;

pub use reexport::*;
