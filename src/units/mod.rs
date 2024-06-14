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
        unit (dimensionless, "") = Dimensionless,
        unit (percent, "%") = 0.01 * Dimensionless,
        def Length = { length: 1 },
        unit (meters, "m") = Length,
        unit (centimeters, "cm") = 0.01 * meters,
        unit (kilometers, "km") = 1000.0 * meters,
        unit (parsec, "pc") = 3.0857e16 * meters,
        unit (kiloparsec, "kpc") = 1000 * parsec,
        unit (megaparsec, "Mpc") = 1000000 * parsec,
        unit (gigaparsec, "Gpc") = 1000000000 * parsec,
        def ComovingLength = { length: 1, a: -1, h: 0 },
        unit (comoving_parsec, "cpc") = 3.0857e16 * ComovingLength,
        unit (comoving_kiloparsec, "ckpc") = 1000 * comoving_parsec,
        unit (comoving_megaparsec, "cMpc") = 1000000 * comoving_parsec,
        unit (comoving_gigaparsec, "cGpc") = 1000000000 * comoving_parsec,
        def ComovingLengthTimesH = ComovingLength * H,
        // This notation is absolute garbage. 
        // People write h^-1 to mean "divide this by h to get the physical quantity"
        // which is against the basic laws of math. Its very inconsistent too. Arepo
        // treats the "h_scaling" in its datasets properly (i.e. you need to multiply by
        // h^-h_scaling to get rid of it.)
        unit (weird_cosmological_notation_parsec, "cpc/h") = comoving_parsec * h,
        unit (weird_cosmological_notation_kiloparsec, "ckpc/h") = comoving_kiloparsec * h,
        unit (weird_cosmological_notation_megaparsec, "cMpc/h") = comoving_megaparsec * h,
        def H = { h: 1 },
        unit (h, "h") = H,
        def A = { a: 1 },
        unit (a, "a") = A,
        def Time = { time: 1 },
        unit (seconds, "s") = 1.0 * Time,
        unit (milliseconds, "ms") = 1e-3 * seconds,
        unit (microseconds, "µs") = 1e-6 * seconds,
        unit (nanoseconds, "ns") = 1e-9 * seconds,
        unit (years, "yr") = 3.15576e7 * seconds,
        unit (kiloyears, "kyr") = 1000.0 * years,
        unit (megayears, "Myr") = 1e6 * years,
        unit (gigayears, "Gyr") = 1e9 * years,
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
        unit (cubic_meters, "m^3") = Volume3D,
        unit (cubic_centimeters, "cm^3") = 1e-6 * cubic_meters,
        def Density = Mass / Volume3D,
        unit (grams_per_cubic_centimeters, "g/cm^3") = grams / cubic_centimeters,
        def Rate = Dimensionless / Time,
        unit (per_second, "s^-1") = 1.0 / seconds,
        def PhotonRate = Rate,
        def SourceRate = Rate,
        unit photons_per_second = 1.0 / seconds,
        def RateDensity = Rate / Volume3D,
        def PhotonRateDensity = PhotonRate / Volume3D,
        unit (photons_per_second_per_cubic_meter, "s^-1 m^-3") = photons_per_second / cubic_meters,
        unit (photons_per_second_per_cubic_centimeter, "s^-1 cm^-3") = photons_per_second / cubic_centimeters,
        def PhotonFlux = PhotonRate / Area,
        unit photons_per_s_per_cm_squared = photons_per_second / centimeters_squared,
        def CrossSection = Area,
        unit (centimeters_squared, "cm^2") = centimeters * centimeters,
        def VolumeRate = Volume3D / Time,
        unit (centimeters_cubed_per_s, "cm^3/s") = cubic_centimeters / seconds,
        def VolumeRateK = Volume3D / (Time * Temperature),
        def HeatingTerm = Energy * Volume3D / Time,
        unit (ergs_centimeters_cubed_per_s, "ergs cm^3/s") = ergs * centimeters_cubed_per_s,
        def HeatingRate = Energy / (Volume3D * Time),
        unit (ergs_per_centimeters_cubed_per_s, "ergs cm^-3 s^-1") = ergs / (cubic_centimeters * seconds),
        def NumberDensity = Dimensionless / Volume3D,
        unit per_centimeters_cubed = 1.0 / cubic_centimeters,

        constant BOLTZMANN_CONSTANT = 1.380649e-23 * joules / kelvins,
        constant PROTON_MASS = 1.67262192369e-27 * kilograms,
        constant SPEED_OF_LIGHT = 299792458.0 * meters_per_second,
        constant GAMMA = 5.0 / 3.0,
        constant NUMBER_WEIGHTED_AVERAGE_CROSS_SECTION = 2.9580524545305314e-18 * centimeters_squared,
        constant ENERGY_WEIGHTED_AVERAGE_CROSS_SECTION = 2.7352520425024469e-18 * centimeters_squared,
        constant PHOTON_AVERAGE_ENERGY = 18.028356312818811 * electron_volts,
        constant RYDBERG_CONSTANT = 13.65693 * electron_volts,
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
