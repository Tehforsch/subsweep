use std::ops::Div;

use super::Chemistry;
use super::Timescale;
use crate::sweep::grid::Cell;
use crate::sweep::site::Site;
use crate::units::Density;
use crate::units::Dimension;
use crate::units::Dimensionless;
use crate::units::HeatingRate;
use crate::units::Length;
use crate::units::NumberDensity;
use crate::units::PhotonRate;
use crate::units::Quantity;
use crate::units::Rate;
use crate::units::Temperature;
use crate::units::Time;
use crate::units::Volume;
use crate::units::VolumeRate;
use crate::units::VolumeRateK;
use crate::units::NUMBER_WEIGHTED_AVERAGE_CROSS_SECTION;
use crate::units::PHOTON_AVERAGE_ENERGY;
use crate::units::PROTON_MASS;
use crate::units::RYDBERG_CONSTANT;

const HYDROGEN_MASS_FRACTION: f64 = 1.0;

const MAX_DEPTH: usize = 100;

/// The ionized hydrogen fraction is always kept between this value and (1 - this value)
/// to ensure numerical stability.
const IONIZED_HYDROGEN_FRACTION_EPSILON: f64 = 1e-10;

#[derive(Debug)]
pub struct HydrogenOnly {
    pub rate_threshold: PhotonRate,
    pub scale_factor: Dimensionless,
    pub timestep_safety_factor: Dimensionless,
    pub prevent_cooling: bool,
    pub limit_absorption: bool,
}

#[derive(Debug)]
pub struct HydrogenOnlySpecies {
    pub ionized_hydrogen_fraction: Dimensionless,
    pub temperature: Temperature,
    pub timestep: Time,
}

impl HydrogenOnlySpecies {
    pub(crate) fn new(
        ionized_hydrogen_fraction: Dimensionless,
        temperature: Temperature,
    ) -> HydrogenOnlySpecies {
        Self {
            ionized_hydrogen_fraction,
            temperature,
            timestep: Time::zero(),
        }
    }
}

impl Chemistry for HydrogenOnly {
    type Photons = PhotonRate;
    type Species = HydrogenOnlySpecies;

    fn get_outgoing_rate(
        &self,
        cell: &Cell,
        site: &Site<Self>,
        incoming_rate: Self::Photons,
    ) -> PhotonRate {
        let neutral_hydrogen_number_density =
            site.density / PROTON_MASS * (1.0 - site.species.ionized_hydrogen_fraction);
        let sigma = NUMBER_WEIGHTED_AVERAGE_CROSS_SECTION;
        if incoming_rate < self.rate_threshold {
            PhotonRate::zero()
        } else {
            let non_absorbed_fraction =
                (-neutral_hydrogen_number_density * sigma * cell.size).exp();
            incoming_rate * non_absorbed_fraction
        }
    }

    fn update_abundances(
        &self,
        site: &mut Site<Self>,
        rate: Self::Photons,
        timestep: Time,
        volume: Volume,
        length: Length,
    ) -> Timescale {
        let floor = Some((
            site.species.temperature,
            site.species.ionized_hydrogen_fraction,
        ))
        .filter(|_| self.prevent_cooling);
        let mut solver = Solver {
            ionized_hydrogen_fraction: site.species.ionized_hydrogen_fraction,
            temperature: site.species.temperature,
            density: site.density,
            volume,
            length,
            rate,
            _scale_factor: self.scale_factor,
            floor,
        };
        let timestep_used = solver.perform_timestep(timestep, self.timestep_safety_factor);
        site.species.temperature = solver.temperature;
        site.species.ionized_hydrogen_fraction = solver.ionized_hydrogen_fraction;
        site.species.timestep = timestep_used.time;
        // Timescale of change
        timestep_used
    }
}

struct TimestepCriterionViolated;
struct TimestepConvergenceFailed;

#[derive(Debug)]
pub(crate) struct Solver {
    pub ionized_hydrogen_fraction: Dimensionless,
    pub temperature: Temperature,
    pub density: Density,
    pub volume: Volume,
    pub length: Length,
    pub rate: PhotonRate,
    pub _scale_factor: Dimensionless,
    pub floor: Option<(Temperature, Dimensionless)>,
}

// All numbers taken from Rosdahl et al (2015)
impl Solver {
    fn hydrogen_number_density(&self) -> NumberDensity {
        self.density / PROTON_MASS
    }

    pub fn ionized_hydrogen_number_density(&self) -> NumberDensity {
        self.hydrogen_number_density() * self.ionized_hydrogen_fraction
    }

    pub fn neutral_hydrogen_number_density(&self) -> NumberDensity {
        self.hydrogen_number_density() * (1.0 - self.ionized_hydrogen_fraction)
    }

    pub fn electron_number_density(&self) -> NumberDensity {
        // Assumes zero helium
        self.ionized_hydrogen_number_density()
    }

    fn mu(&self) -> Dimensionless {
        // Holds for hydrogen only
        1.0 / (self.ionized_hydrogen_fraction + 1.0)
    }

    fn collision_fit_function(&self) -> f64 {
        let temperature = self.temperature.in_kelvins();
        temperature.sqrt() / (1.0 + (temperature / 1e5).sqrt()) * (-157809.1 / temperature).exp()
    }

    pub fn case_b_recombination_rate(&self) -> VolumeRate {
        let lambda = Temperature::kelvins(315614.0) / self.temperature;
        VolumeRate::centimeters_cubed_per_s(
            2.753e-14 * lambda.powf(1.5) / (1.0 + (lambda / 2.74).powf(0.407)).powf(2.242),
        )
    }

    pub fn collisional_ionization_rate(&self) -> VolumeRate {
        VolumeRate::centimeters_cubed_per_s(5.85e-11 * self.collision_fit_function())
    }

    pub fn cooling_rate(&self) -> HeatingRate {
        HeatingRate::zero()
    }

    fn temperature_change(&mut self, _: Time) -> Temperature {
        Temperature::zero()
    }

    fn num_newly_ionized_hydrogen_atoms(&self, timestep: Time) -> Dimensionless {
        let neutral_hydrogen_number_density = self.neutral_hydrogen_number_density();
        let sigma = NUMBER_WEIGHTED_AVERAGE_CROSS_SECTION;
        let absorbed_fraction =
            1.0 - (-neutral_hydrogen_number_density * sigma * self.length).exp();
        let num_photons: Dimensionless = timestep * self.rate;
        num_photons * absorbed_fraction
    }

    pub fn photoheating_rate(&self, timestep: Time) -> HeatingRate {
        let num_ionized_hydrogen_atoms = self.num_newly_ionized_hydrogen_atoms(timestep);
        let ionization_density = num_ionized_hydrogen_atoms / self.volume;
        ionization_density * (PHOTON_AVERAGE_ENERGY - RYDBERG_CONSTANT) / timestep
    }

    pub fn photoionization_rate(&self, timestep: Time) -> Rate {
        let num_ionized_hydrogen_atoms = self.num_newly_ionized_hydrogen_atoms(timestep);
        let fraction_ionized_hydrogen_atoms =
            num_ionized_hydrogen_atoms / (self.neutral_hydrogen_number_density() * self.volume);
        fraction_ionized_hydrogen_atoms / timestep
    }

    fn ionized_fraction_change(&mut self, timestep: Time) -> Dimensionless {
        // See A23 of Rosdahl et al
        let nh = self.hydrogen_number_density();
        let ne = self.electron_number_density();
        let alpha: VolumeRate = VolumeRate::zero();
        let dalpha: VolumeRateK = VolumeRate::zero() / Temperature::kelvins(1.0);
        let beta: VolumeRate = VolumeRate::zero();
        let dbeta: VolumeRateK = VolumeRate::zero() / Temperature::kelvins(1.0);
        let photoionization_rate = self.photoionization_rate(timestep);
        let c: Rate = beta * ne + photoionization_rate;
        let mu = self.mu();
        let d: Rate = alpha * ne;
        let xhii = self.ionized_hydrogen_fraction;
        // Derivative
        let rhsc: Rate = ne * self.temperature * mu * HYDROGEN_MASS_FRACTION * dbeta;
        let dcdx: Rate = nh * beta - rhsc;
        let rhsd: Rate = ne * self.temperature * mu * HYDROGEN_MASS_FRACTION * dalpha;
        let dddx: Rate = nh * alpha - rhsd;
        let j = dcdx - (c + d) - xhii * (dcdx + dddx);
        timestep * (c - xhii * (c + d)) / (1.0 - j * timestep)
    }

    fn clamp(&mut self) {
        let xhii_floor = self
            .floor
            .map(|(_, xhii)| *xhii)
            .unwrap_or(IONIZED_HYDROGEN_FRACTION_EPSILON);
        self.ionized_hydrogen_fraction = self
            .ionized_hydrogen_fraction
            .clamp(xhii_floor, 1.0 - IONIZED_HYDROGEN_FRACTION_EPSILON);
        if let Some((temp_floor, _)) = self.floor {
            if self.temperature < temp_floor {
                self.temperature = temp_floor;
            }
        }
    }

    fn try_timestep_update(
        &mut self,
        timestep: Time,
        timestep_safety_factor: Dimensionless,
    ) -> Result<Timescale, TimestepCriterionViolated> {
        let temperature_change = self.temperature_change(timestep);
        let ideal_temperature_timestep = Timescale::temperature(update(
            &mut self.temperature,
            temperature_change,
            timestep_safety_factor,
            timestep,
        )?);
        let ionized_fraction_change = self.ionized_fraction_change(timestep);
        let ideal_ionized_fraction_timestep = Timescale::ionization_fraction(update(
            &mut self.ionized_hydrogen_fraction,
            ionized_fraction_change,
            timestep_safety_factor,
            timestep,
        )?);
        self.clamp();
        Ok(ideal_temperature_timestep.min(ideal_ionized_fraction_timestep))
    }

    fn perform_timestep_internal(
        &mut self,
        timestep: Time,
        timestep_safety_factor: Dimensionless,
        depth: usize,
        max_depth: usize,
    ) -> Result<Timescale, TimestepConvergenceFailed> {
        self.clamp();
        let initial_state = (self.temperature, self.ionized_hydrogen_fraction);
        if depth > max_depth {
            return Err(TimestepConvergenceFailed);
        }
        match self.try_timestep_update(timestep, timestep_safety_factor) {
            Err(TimestepCriterionViolated) => {
                (self.temperature, self.ionized_hydrogen_fraction) = initial_state;
                self.perform_timestep_internal(
                    timestep / 2.0,
                    timestep_safety_factor,
                    depth + 1,
                    max_depth,
                )?;
                self.perform_timestep_internal(
                    timestep / 2.0,
                    timestep_safety_factor,
                    depth + 1,
                    max_depth,
                )
            }
            Ok(timestep_recommendation) => Ok(timestep_recommendation),
        }
    }

    pub fn perform_timestep(
        &mut self,
        timestep: Time,
        timestep_safety_factor: Dimensionless,
    ) -> Timescale {
        self.perform_timestep_internal(timestep, timestep_safety_factor, 0, MAX_DEPTH)
            .unwrap_or_else(|_| {
                log::error!(
                    "Failed to find timestep in chemistry. Solver state: {:?}",
                    self
                );
                // We don't panic here to make sure we can still run
                // the process but lets return a pessimistic timescale
                Timescale::temperature(timestep / 10.0)
            })
    }
}

fn update<const D: Dimension>(
    value: &mut Quantity<f64, D>,
    change: Quantity<f64, D>,
    max_allowed_change: Dimensionless,
    timestep: Time,
) -> Result<Time, TimestepCriterionViolated>
where
    Quantity<f64, D>: Div<Quantity<f64, D>, Output = Dimensionless>,
{
    let relative_change = (change / *value).abs().min(1.0 / f64::EPSILON);
    if relative_change > max_allowed_change {
        Err(TimestepCriterionViolated)
    } else {
        *value += change;
        let timestep_recommendation = timestep * (max_allowed_change / relative_change);
        Ok(timestep_recommendation)
    }
}
