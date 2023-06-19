use std::ops::Div;

use diman::Quotient;

use super::Chemistry;
use crate::sweep::grid::Cell;
use crate::sweep::site::Site;
use crate::units::Density;
use crate::units::Dimension;
use crate::units::Dimensionless;
use crate::units::EnergyPerTime;
use crate::units::HeatingRate;
use crate::units::HeatingTerm;
use crate::units::InverseTemperature;
use crate::units::Length;
use crate::units::NumberDensity;
use crate::units::PhotonRate;
use crate::units::Quantity;
use crate::units::Rate;
use crate::units::Temperature;
use crate::units::Time;
use crate::units::Volume;
use crate::units::VolumeRate;
use crate::units::BOLTZMANN_CONSTANT;
use crate::units::ENERGY_WEIGHTED_AVERAGE_CROSS_SECTION;
use crate::units::GAMMA;
use crate::units::NUMBER_WEIGHTED_AVERAGE_CROSS_SECTION;
use crate::units::PHOTON_AVERAGE_ENERGY;
use crate::units::PROTON_MASS;
use crate::units::RYDBERG_CONSTANT;
use crate::units::SPEED_OF_LIGHT;

const HYDROGEN_MASS_FRACTION: f64 = 1.0;

const MAX_DEPTH: usize = 10;

pub struct HydrogenOnly {
    pub rate_threshold: PhotonRate,
    pub scale_factor: Dimensionless,
    pub timestep_safety_factor: Dimensionless,
}

#[derive(Debug)]
pub struct HydrogenOnlySpecies {
    pub ionized_hydrogen_fraction: Dimensionless,
    pub temperature: Temperature,
    pub heating_rate: HeatingRate,
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
            heating_rate: HeatingRate::zero(),
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
            let absorbed_fraction = (-neutral_hydrogen_number_density * sigma * cell.size).exp();
            incoming_rate * absorbed_fraction
        }
    }

    fn update_abundances(
        &self,
        site: &mut Site<Self>,
        rate: Self::Photons,
        timestep: Time,
        volume: Volume,
        length: Length,
    ) -> Time {
        let mut solver = Solver {
            ionized_hydrogen_fraction: site.species.ionized_hydrogen_fraction,
            temperature: site.species.temperature,
            density: site.density,
            volume,
            length,
            rate,
            scale_factor: self.scale_factor,
            heating_rate: HeatingRate::zero(),
        };
        let timestep_used = solver.perform_timestep(timestep, self.timestep_safety_factor);
        site.species.temperature = solver.temperature;
        site.species.ionized_hydrogen_fraction = solver.ionized_hydrogen_fraction;
        site.species.heating_rate = solver.heating_rate;
        site.species.timestep = timestep_used;
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
    pub scale_factor: Dimensionless,
    pub heating_rate: HeatingRate,
}

// All numbers taken from Rosdahl et al (2015)
impl Solver {
    fn hydrogen_number_density(&self) -> NumberDensity {
        self.density / PROTON_MASS
    }

    fn ionized_hydrogen_number_density(&self) -> NumberDensity {
        self.hydrogen_number_density() * self.ionized_hydrogen_fraction
    }

    fn neutral_hydrogen_number_density(&self) -> NumberDensity {
        self.hydrogen_number_density() * (1.0 - self.ionized_hydrogen_fraction)
    }

    fn electron_number_density(&self) -> NumberDensity {
        // Assumes zero helium
        self.ionized_hydrogen_number_density()
    }

    fn mu(&self) -> Dimensionless {
        // Holds for hydrogen only
        1.0 / (self.ionized_hydrogen_fraction + 1.0)
    }

    fn photon_density(&self) -> NumberDensity {
        self.rate * self.length / SPEED_OF_LIGHT / self.volume
    }

    fn collision_fit_function(&self) -> f64 {
        let temperature = self.temperature.in_kelvins();
        temperature.sqrt() / (1.0 + (temperature / 1e5).sqrt()) * (-157809.1 / temperature).exp()
    }

    fn collision_fit_function_derivative(&self) -> f64 {
        let const1 = 1.0 / 1e5;
        let const2 = 157809.1;
        let t = self.temperature.in_kelvins();
        ((-const2 / t).exp()
            * (const1 * const2 * t + 0.5 * (const1 * t).sqrt() * (2.0 * const2 + t)))
            / (t.powi(3).sqrt() * (const1 * t).sqrt() * ((const1 * t).sqrt() + 1.0).powi(2))
    }

    fn case_b_recombination_rate(&self) -> VolumeRate {
        let lambda = Temperature::kelvins(315614.0) / self.temperature;
        VolumeRate::centimeters_cubed_per_s(
            2.753e-14 * lambda.powf(1.5) / (1.0 + (lambda / 2.74).powf(0.407)).powf(2.242),
        )
    }

    fn case_b_recombination_rate_derivative(&self) -> Quotient<VolumeRate, Temperature> {
        let lambda = (Temperature::kelvins(315614.0) / self.temperature).value();
        let dlambda_dt: InverseTemperature =
            -Temperature::kelvins(315614.0) / self.temperature.squared();
        let c1 = 1.0 / 2.74;
        let c2 = 0.407;
        let c3 = 2.242;
        let d = -lambda.sqrt()
            * ((c1 * lambda).powf(c2) + 1.0).powf(-c3 - 1.0)
            * (c2 * c3 * (c1 * lambda).powf(c2) - 1.5 * (c1 * lambda).powf(c2) - 1.5);

        VolumeRate::centimeters_cubed_per_s(2.753e-14 * d) * dlambda_dt
    }

    fn case_b_recombination_cooling_rate(&self) -> HeatingTerm {
        let lambda = Temperature::kelvins(315614.0) / self.temperature;
        HeatingTerm::ergs_centimeters_cubed_per_s(
            3.435e-30 * self.temperature.in_kelvins() * lambda.powf(1.97)
                / (1.0 + (lambda / 2.25).powf(0.376)).powf(3.72),
        )
    }

    fn case_b_recombination_cooling_rate_derivative(&self) -> Quotient<HeatingTerm, Temperature> {
        let c1 = 315614.0;
        let c2 = 1.97;
        let c3 = 0.376;
        let c4 = 3.72;
        let c5 = 2.25;
        let t = self.temperature.in_kelvins();
        let derivative = (1.0 + (c1 / (c5 * t)).powf(c3)).powf(-1.0 - c4)
            * (1.0 - 1.0 * c2 + (1.0 - 1.0 * c2 + c3 * c4) * (c1 / (c5 * t)).powf(c3))
            * (c1 / t).powf(c2);
        HeatingTerm::ergs_centimeters_cubed_per_s(3.435e-30 * derivative)
            / Temperature::kelvins(1.0)
    }

    fn collisional_ionization_rate(&self) -> VolumeRate {
        VolumeRate::centimeters_cubed_per_s(5.85e-11 * self.collision_fit_function())
    }

    fn collisional_ionization_rate_derivative(&self) -> Quotient<VolumeRate, Temperature> {
        VolumeRate::centimeters_cubed_per_s(5.85e-11 * self.collision_fit_function_derivative())
            / Temperature::kelvins(1.0)
    }

    fn collisional_ionization_cooling_rate(&self) -> HeatingTerm {
        HeatingTerm::ergs_centimeters_cubed_per_s(1.27e-21 * self.collision_fit_function())
    }

    fn collisional_ionization_cooling_rate_derivative(&self) -> Quotient<HeatingTerm, Temperature> {
        HeatingTerm::ergs_centimeters_cubed_per_s(
            1.27e-21 * self.collision_fit_function_derivative(),
        ) / Temperature::kelvins(1.0)
    }

    fn collisional_excitation_cooling_rate(&self) -> HeatingTerm {
        let temperature = self.temperature.in_kelvins();
        HeatingTerm::ergs_centimeters_cubed_per_s(
            7.5e-19 / (1.0 + (temperature / 1e5).sqrt()) * (-118348.0 / temperature).exp(),
        )
    }

    fn collisional_excitation_cooling_rate_derivative(&self) -> Quotient<HeatingTerm, Temperature> {
        let t = self.temperature.in_kelvins();
        let c1 = 7.5e-19;
        let c2 = 118348.0;
        let c3 = 1.0 / 1e5;
        HeatingTerm::ergs_centimeters_cubed_per_s(
            (c1 * (-c2 / t).exp() * (c2 * c3 * t - 0.5 * c3 * t.powi(2) + c2 * (c3 * t).sqrt()))
                / (t.powi(2) * (c3 * t).sqrt() * (1.0 + (c3 * t).sqrt()).powi(2)),
        ) / Temperature::kelvins(1.0)
    }

    fn bremsstrahlung_cooling_rate(&self) -> HeatingTerm {
        HeatingTerm::ergs_centimeters_cubed_per_s(1.42e-27 * self.temperature.in_kelvins().sqrt())
    }

    fn bremsstrahlung_cooling_rate_derivative(&self) -> Quotient<HeatingTerm, Temperature> {
        HeatingTerm::ergs_centimeters_cubed_per_s(
            1.42e-27 / (2.0 * self.temperature.in_kelvins().sqrt()),
        ) / Temperature::kelvins(1.0)
    }

    fn compton_cooling_rate(&self) -> EnergyPerTime {
        let x = (2.727 / self.scale_factor).value();
        EnergyPerTime::ergs_per_s(1.017e-37 * x.powi(4) * (self.temperature.in_kelvins() - x))
    }

    fn compton_cooling_rate_derivative(&self) -> Quotient<EnergyPerTime, Temperature> {
        let x = (2.727 / self.scale_factor).value();
        EnergyPerTime::ergs_per_s(1.017e-37 * x.powi(4)) / Temperature::kelvins(1.0)
    }

    fn photoheating_rate(&self) -> HeatingRate {
        self.neutral_hydrogen_number_density()
            * self.photon_density()
            * SPEED_OF_LIGHT
            * (PHOTON_AVERAGE_ENERGY * ENERGY_WEIGHTED_AVERAGE_CROSS_SECTION
                - RYDBERG_CONSTANT * NUMBER_WEIGHTED_AVERAGE_CROSS_SECTION)
    }

    fn cooling_rate(&self) -> HeatingRate {
        let ne = self.electron_number_density();
        let nh_neutral = self.neutral_hydrogen_number_density();
        let nh_ionized = self.ionized_hydrogen_number_density();
        let collisional = (self.collisional_excitation_cooling_rate()
            + self.collisional_ionization_cooling_rate())
            * ne
            * nh_neutral;
        let recombination = self.case_b_recombination_cooling_rate() * ne * nh_ionized;
        let bremsstrahlung = self.bremsstrahlung_cooling_rate() * ne * nh_ionized;
        let compton: HeatingRate = self.compton_cooling_rate() * ne;
        collisional + recombination + bremsstrahlung + compton
    }

    fn cooling_rate_derivative(&self) -> Quotient<HeatingRate, Temperature> {
        let ne = self.electron_number_density();
        let nh_neutral = self.neutral_hydrogen_number_density();
        let nh_ionized = self.ionized_hydrogen_number_density();
        let collisional = (self.collisional_excitation_cooling_rate_derivative()
            + self.collisional_ionization_cooling_rate_derivative())
            * ne
            * nh_neutral;
        let recombination = self.case_b_recombination_cooling_rate_derivative() * ne * nh_ionized;
        let bremsstrahlung = self.bremsstrahlung_cooling_rate_derivative() * ne * nh_ionized;
        let compton: Quotient<HeatingRate, Temperature> =
            self.compton_cooling_rate_derivative() * ne;
        collisional + recombination + bremsstrahlung + compton
    }

    fn temperature_change(&mut self, timestep: Time) -> Temperature {
        let k = (GAMMA - 1.0) * PROTON_MASS / (self.density * BOLTZMANN_CONSTANT);
        let lambda = self.photoheating_rate() - self.cooling_rate();
        let dlambdadt = -self.cooling_rate_derivative();
        self.heating_rate = lambda;
        let mu = self.mu();
        k * mu * lambda * timestep / (1.0 - k * mu * dlambdadt * timestep)
    }

    fn photoionization_rate(&self) -> Rate {
        NUMBER_WEIGHTED_AVERAGE_CROSS_SECTION * SPEED_OF_LIGHT * self.photon_density()
    }

    fn ionized_fraction_change(&self, timestep: Time) -> Dimensionless {
        // See A23 of Rosdahl et al
        let nh = self.hydrogen_number_density();
        let ne = self.electron_number_density();
        let alpha = self.case_b_recombination_rate();
        let dalpha = self.case_b_recombination_rate_derivative();
        let beta = self.collisional_ionization_rate();
        let dbeta = self.collisional_ionization_rate_derivative();
        let c: Rate = beta * ne + self.photoionization_rate();
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

    fn try_timestep_update(
        &mut self,
        timestep: Time,
        timestep_safety_factor: Dimensionless,
    ) -> Result<Time, TimestepCriterionViolated> {
        let temperature_change = self.temperature_change(timestep);
        let ideal_temperature_timestep = update(
            &mut self.temperature,
            temperature_change,
            timestep_safety_factor,
            timestep,
        )?;
        let ionized_fraction_change = self.ionized_fraction_change(timestep);
        let ideal_ionized_fraction_timestep = update(
            &mut self.ionized_hydrogen_fraction,
            ionized_fraction_change,
            timestep_safety_factor,
            timestep,
        )?;
        self.ionized_hydrogen_fraction = self.ionized_hydrogen_fraction.clamp(0.0, 1.0);
        Ok(ideal_temperature_timestep.min(ideal_ionized_fraction_timestep))
    }

    fn perform_timestep_internal(
        &mut self,
        timestep: Time,
        timestep_safety_factor: Dimensionless,
        depth: usize,
        max_depth: usize,
    ) -> Result<Time, TimestepConvergenceFailed> {
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
    ) -> Time {
        self.perform_timestep_internal(timestep, timestep_safety_factor, 0, MAX_DEPTH)
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to find timestep in chemistry. Solver state: {:?}",
                    self
                )
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

#[cfg(not(feature = "2d"))]
#[cfg(test)]
mod tests {
    use std::fs;
    use std::ops::Div;
    use std::ops::Mul;
    use std::ops::Sub;
    use std::path::Path;

    use super::Solver;
    use crate::units::Density;
    use crate::units::Dimension;
    use crate::units::Dimensionless;
    use crate::units::HeatingRate;
    use crate::units::Length;
    use crate::units::NumberDensity;
    use crate::units::PhotonFlux;
    use crate::units::Quantity;
    use crate::units::Rate;
    use crate::units::Temperature;
    use crate::units::Time;
    use crate::units::Volume;
    use crate::units::PROTON_MASS;

    #[allow(unused)]
    const MAX_ALLOWED_RELATIVE_CHANGE: f64 = 0.01;

    fn test_numerical_derivative<const D1: Dimension, const D2: Dimension>(
        function: fn(&Solver) -> Quantity<f64, D1>,
        derivative: fn(&Solver) -> Quantity<f64, D2>,
    ) where
        Quantity<f64, D1>: Mul<Quantity<f64, D1>>,
        Quantity<f64, D1>: Sub<Quantity<f64, D1>, Output = Quantity<f64, D1>>,
        Quantity<f64, D1>: Div<Temperature, Output = Quantity<f64, D2>>,
        Quantity<f64, D2>: Div<Quantity<f64, D2>, Output = Dimensionless>,
    {
        // We can't have very high standards for accuracy here because numerical precision is
        // terrible
        let epsilon = 1e-1;
        let delta = Temperature::kelvins(1e-6);
        for temperature in [
            Temperature::kelvins(1e1),
            Temperature::kelvins(1e2),
            Temperature::kelvins(1e3),
            Temperature::kelvins(1e4),
            Temperature::kelvins(1e5),
            Temperature::kelvins(1e6),
            Temperature::kelvins(1e7),
        ] {
            let mut solver = Solver {
                temperature,
                // none of these matter
                ionized_hydrogen_fraction: Dimensionless::zero(),
                density: Density::zero(),
                volume: Volume::zero(),
                length: Length::zero(),
                rate: Rate::zero(),
                heating_rate: HeatingRate::zero(),
                scale_factor: Dimensionless::dimensionless(1.0),
            };
            let analytical = derivative(&solver);
            let v1 = function(&solver);
            solver.temperature += delta;
            let v2 = function(&solver);
            let numerical = (v2 - v1) / delta;
            assert!(
                (analytical - numerical).abs()
                    / (analytical.abs() + numerical.abs() + Quantity::new_unchecked(1e-50))
                    < epsilon
            );
        }
    }

    #[test]
    fn case_b_recombination_rate_derivative() {
        test_numerical_derivative(
            Solver::case_b_recombination_rate,
            Solver::case_b_recombination_rate_derivative,
        )
    }

    #[test]
    fn collisional_ionization_rate_derivative() {
        test_numerical_derivative(
            Solver::collisional_ionization_rate,
            Solver::collisional_ionization_rate_derivative,
        )
    }

    #[test]
    fn collisional_ionization_cooling_rate_derivative() {
        test_numerical_derivative(
            Solver::collisional_ionization_cooling_rate,
            Solver::collisional_ionization_cooling_rate_derivative,
        )
    }

    #[test]
    fn collisional_excitation_cooling_rate_derivative() {
        test_numerical_derivative(
            Solver::collisional_excitation_cooling_rate,
            Solver::collisional_excitation_cooling_rate_derivative,
        )
    }

    #[test]
    fn case_b_recombination_cooling_rate_derivative() {
        test_numerical_derivative(
            Solver::case_b_recombination_cooling_rate,
            Solver::case_b_recombination_cooling_rate_derivative,
        )
    }

    #[test]
    fn bremsstrahlung_cooling_rate_derivative() {
        test_numerical_derivative(
            Solver::bremsstrahlung_cooling_rate,
            Solver::bremsstrahlung_cooling_rate_derivative,
        )
    }

    #[test]
    fn compton_cooling_rate_derivative() {
        test_numerical_derivative(
            Solver::compton_cooling_rate,
            Solver::compton_cooling_rate_derivative,
        )
    }

    struct Configuration {
        init_xhii: Dimensionless,
        flux: PhotonFlux,
        temperature: Temperature,
        density: Density,
        modifier: fn(&mut Solver, &Configuration),
        final_time: Time,
        output_times: Vec<Time>,
    }

    struct State {
        xhii: Dimensionless,
        temperature: Temperature,
        time: Time,
        recombination: HeatingRate,
        bremsstrahlung: HeatingRate,
        compton: HeatingRate,
        collisional_excitation: HeatingRate,
        collisional_ionization: HeatingRate,
        ne: NumberDensity,
        nh_neutral: NumberDensity,
    }

    impl Configuration {
        fn new(
            init_xhii: Dimensionless,
            flux: PhotonFlux,
            temperature: Temperature,
            density: Density,
            final_time: Time,
            modifier: fn(&mut Solver, &Configuration),
        ) -> Self {
            let num_outputs = 1000;
            let output_min_exp = -1.0;
            let output_max_exp = final_time.in_megayears().log10() + 0.01;
            let output_times: Vec<_> = (0..num_outputs)
                .map(|i| {
                    let exp = output_min_exp
                        + ((output_max_exp - output_min_exp) / num_outputs as f64 * i as f64);
                    Time::megayears(10.0f64.powf(exp))
                })
                .collect();
            Self {
                init_xhii,
                flux,
                temperature,
                density,
                modifier,
                final_time,
                output_times,
            }
        }

        fn get_solver(&self) -> Solver {
            let length = Length::parsec(1.0);
            let volume = length.cubed();
            let area = volume / length;
            let rate = self.flux * area;

            Solver {
                ionized_hydrogen_fraction: self.init_xhii,
                temperature: self.temperature,
                density: self.density,
                volume,
                length,
                rate,
                scale_factor: Dimensionless::dimensionless(1.0),
                heating_rate: HeatingRate::zero(),
            }
        }

        fn perform_timestep(&self, solver: &mut Solver, timestep: Time, depth: usize) {
            let initial_state = (solver.temperature, solver.ionized_hydrogen_fraction);
            (self.modifier)(solver, self);
            if let Err(_) = solver.try_timestep_update(timestep, Dimensionless::dimensionless(0.1))
            {
                (solver.temperature, solver.ionized_hydrogen_fraction) = initial_state;
                (self.modifier)(solver, self);
                self.perform_timestep(solver, timestep / 2.0, depth + 1);
                self.perform_timestep(solver, timestep / 2.0, depth + 1);
            }
            (self.modifier)(solver, self);
        }

        fn get_state(&self, time: Time, solver: &Solver) -> State {
            let ne = solver.electron_number_density();
            let nh_neutral = solver.neutral_hydrogen_number_density();
            let nh_ionized = solver.ionized_hydrogen_number_density();
            let recombination = solver.case_b_recombination_cooling_rate() * ne * nh_ionized;
            let bremsstrahlung = solver.bremsstrahlung_cooling_rate() * ne * nh_ionized;
            let compton: HeatingRate = solver.compton_cooling_rate() * ne;
            let collisional_excitation =
                solver.collisional_excitation_cooling_rate() * ne * nh_neutral;
            let collisional_ionization =
                solver.collisional_ionization_cooling_rate() * ne * nh_neutral;
            State {
                xhii: solver.ionized_hydrogen_fraction,
                temperature: solver.temperature,
                time,
                recombination,
                bremsstrahlung,
                compton,
                collisional_excitation,
                collisional_ionization,
                ne,
                nh_neutral,
            }
        }

        fn run(&self) -> Vec<State> {
            let mut solver = self.get_solver();
            let mut states = vec![];
            let mut time = Time::zero();

            let timestep = Time::years(10000.0);
            let mut output_times = self.output_times.iter();
            let mut next_output_time = output_times.next().unwrap();
            while time < self.final_time {
                self.perform_timestep(&mut solver, timestep, 0);
                time += timestep;
                if time > *next_output_time {
                    next_output_time = output_times.next().unwrap();
                    states.push(self.get_state(time, &solver));
                }
            }
            states
        }
    }

    fn get_configurations<'a>(
        flux: PhotonFlux,
        init_xhii: &'a [f64],
        temperature: &'a [Temperature],
        density: &'a [Density],
        final_time: Time,
        modifier: fn(&mut Solver, &Configuration),
    ) -> impl Iterator<Item = Configuration> + 'a {
        init_xhii.iter().flat_map(move |init_xhii| {
            temperature.iter().flat_map(move |temperature| {
                density.iter().map(move |density| {
                    Configuration::new(
                        Dimensionless::dimensionless(*init_xhii),
                        flux,
                        *temperature,
                        *density,
                        final_time,
                        modifier.clone(),
                    )
                })
            })
        })
    }

    fn run_configurations(output_file: &str, configurations: impl Iterator<Item = Configuration>) {
        let output_file = Path::new(output_file).to_owned();
        let output_folder = output_file.parent().unwrap();
        fs::create_dir_all(output_folder).unwrap();
        let mut lines = vec![];
        lines.push(
            "flux,init_xHII,init_T,density,t,xHII,T,recomb,brems,compton,coll_ion,coll_ex,ne,nhxi"
                .into(),
        );
        for config in configurations {
            println!(
                "{:?} {:?} {:?} {:?}",
                config.flux, config.density, config.temperature, config.init_xhii
            );
            let states = config.run();
            lines.extend(states.into_iter().map(|state| {
                format!(
                    "{:+e},{:+e},{:+e},{:+e},{:+e},{:+e},{:+e},{:+e},{:+e},{:+e},{:+e},{:+e},{:+e},{:+e}",
                    config.flux.in_photons_per_s_per_cm_squared(),
                    config.init_xhii.value(),
                    config.temperature.in_kelvins(),
                    (config.density / PROTON_MASS).in_per_centimeters_cubed(),
                    state.time.in_megayears(),
                    state.xhii.value(),
                    state.temperature.in_kelvins(),
                    state.recombination.in_ergs_per_centimeters_cubed_per_s(),
                    state.bremsstrahlung.in_ergs_per_centimeters_cubed_per_s(),
                    state.compton.in_ergs_per_centimeters_cubed_per_s(),
                    state
                        .collisional_ionization
                        .in_ergs_per_centimeters_cubed_per_s(),
                    state
                        .collisional_excitation
                        .in_ergs_per_centimeters_cubed_per_s(),
                    state.ne.in_per_centimeters_cubed(),
                    state.nh_neutral.in_per_centimeters_cubed(),
                )
            }));
        }
        fs::write(output_file, lines.join("\n")).unwrap();
    }

    fn as_density(d: f64) -> Density {
        let number_density = NumberDensity::per_centimeters_cubed(d);
        number_density * PROTON_MASS
    }

    fn reset_temp(solver: &mut Solver, config: &Configuration) {
        solver.temperature = config.temperature;
    }

    fn reset_xhii(solver: &mut Solver, config: &Configuration) {
        solver.ionized_hydrogen_fraction = config.init_xhii;
    }

    fn do_nothing(_: &mut Solver, _: &Configuration) {}

    #[test]
    #[ignore]
    fn time_evolution_const_xhii() {
        for (name, flux) in [
            (
                "out/const_xhii",
                PhotonFlux::photons_per_s_per_cm_squared(0.0),
            ),
            (
                "out/const_xhii_flux",
                PhotonFlux::photons_per_s_per_cm_squared(1e5),
            ),
        ] {
            run_configurations(
                name,
                get_configurations(
                    flux,
                    &[1e-10, 0.2, 0.5, 0.8, 1.0],
                    &[
                        Temperature::kelvins(1e3),
                        Temperature::kelvins(1.6e4),
                        Temperature::kelvins(1.8e5),
                        Temperature::kelvins(3e6),
                        Temperature::kelvins(1e8),
                    ],
                    &[
                        as_density(1e-8),
                        as_density(1e-6),
                        as_density(1e-4),
                        as_density(1e-2),
                        as_density(1e0),
                        as_density(1e2),
                    ],
                    Time::megayears(1e4),
                    reset_xhii,
                ),
            );
        }
    }
    #[test]
    #[ignore]
    fn time_evolution_const_temp() {
        for (name, flux) in [
            (
                "out/const_temp",
                PhotonFlux::photons_per_s_per_cm_squared(0.0),
            ),
            (
                "out/const_temp_flux",
                PhotonFlux::photons_per_s_per_cm_squared(1e5),
            ),
        ] {
            run_configurations(
                name,
                get_configurations(
                    flux,
                    &[1e-10, 0.2, 0.5, 0.8, 1.0],
                    &[
                        Temperature::kelvins(1e3),
                        Temperature::kelvins(1.6e4),
                        Temperature::kelvins(3e4),
                        Temperature::kelvins(9e4),
                        Temperature::kelvins(1e6),
                    ],
                    &[
                        as_density(1e-8),
                        as_density(1e-6),
                        as_density(1e-4),
                        as_density(1e-2),
                        as_density(1e0),
                        as_density(1e2),
                    ],
                    Time::megayears(1e4),
                    reset_temp,
                ),
            );
        }
    }

    #[test]
    #[ignore]
    fn time_evolution_free() {
        for (name, flux) in [
            (
                "out/evolution",
                PhotonFlux::photons_per_s_per_cm_squared(0.0),
            ),
            (
                "out/evolution_flux",
                PhotonFlux::photons_per_s_per_cm_squared(1e5),
            ),
        ] {
            run_configurations(
                name,
                get_configurations(
                    flux,
                    &[1e-10, 0.2, 0.5, 0.8, 1.0],
                    &[
                        Temperature::kelvins(1e3),
                        Temperature::kelvins(1.6e4),
                        Temperature::kelvins(3e4),
                        Temperature::kelvins(9e4),
                        Temperature::kelvins(1e6),
                    ],
                    &[
                        as_density(1e-8),
                        as_density(1e-6),
                        as_density(1e-4),
                        as_density(1e-2),
                        as_density(1e0),
                        as_density(1e2),
                    ],
                    Time::megayears(1e4),
                    do_nothing,
                ),
            );
        }
    }

    #[test]
    #[ignore]
    fn rates_over_time() {
        let name = "out/rates";
        let dens = as_density(1e-2);
        let num_temps = 160;
        let temperatures: Vec<_> = (0..num_temps)
            .map(|i| {
                let exp = 4.0 + (0.025 * i as f64);
                Temperature::kelvins(10.0f64.powf(exp))
            })
            .collect();
        let flux = PhotonFlux::zero();
        run_configurations(
            name,
            get_configurations(
                flux,
                &[0.5],
                &temperatures,
                &[dens],
                Time::megayears(5000.0),
                reset_temp,
            ),
        );
    }
}
