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
use crate::units::PROTON_MASS;
use crate::units::SPEED_OF_LIGHT;
use crate::units::SWEEP_HYDROGEN_ONLY_AVERAGE_PHOTON_ENERGY;
use crate::units::SWEEP_HYDROGEN_ONLY_CROSS_SECTION;

const HYDROGEN_MASS_FRACTION: f64 = 1.0;

const MAX_DEPTH: usize = 100;

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
        let sigma = crate::units::SWEEP_HYDROGEN_ONLY_CROSS_SECTION;
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
        self.density / PROTON_MASS * self.ionized_hydrogen_fraction
    }

    fn electron_number_density(&self) -> NumberDensity {
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

    fn collision_fit_function_derivative(&self) -> f64 {
        let const1 = 1e5;
        let const2 = 157809.1;
        let t = self.temperature.in_kelvins();
        (-const2 / t).exp() * (t / const1).powi(2).sqrt()
            / (2.0 * t.sqrt() * ((t / const1).sqrt() + 1.0).powi(2))
    }

    fn case_b_recombination_rate(&self) -> VolumeRate {
        let lambda = Temperature::kelvins(315614.0) / self.temperature;
        VolumeRate::centimeters_cubed_per_s(
            2.753e-14 * lambda.powf(1.5) / (1.0 + (lambda / 2.74).powf(0.407)).powf(2.242),
        )
    }

    fn case_b_recombination_rate_derivative(&self) -> Quotient<VolumeRate, Temperature> {
        let lambda = Temperature::kelvins(315614.0) / self.temperature;
        let dlambda_dt: InverseTemperature =
            -Temperature::kelvins(315614.0) / self.temperature.squared();
        let c1 = 2.242;
        let c2 = 0.407;
        let c3 = 2.74;

        VolumeRate::centimeters_cubed_per_s(
            2.753e-14
                * lambda.powf(1.5)
                * ((lambda / c3).powf(c2) + 1.0).powf(-c1)
                * ((lambda / c3).powf(c2) + 1.0).ln(),
        ) * dlambda_dt
    }

    fn case_b_recombination_cooling_rate(&self) -> HeatingTerm {
        let lambda = Temperature::kelvins(315614.0) / self.temperature;
        HeatingTerm::ergs_centimeters_cubed_per_s(
            3.435e-30 * self.temperature.in_kelvins() * lambda.powf(1.97)
                / (1.0 + (lambda / 2.25).powf(0.376)).powf(3.72),
        )
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

    fn collisional_excitation_cooling_rate(&self) -> HeatingTerm {
        let temperature = self.temperature.in_kelvins();
        HeatingTerm::ergs_centimeters_cubed_per_s(
            7.5e-19 / (1.0 + (temperature / 1e5).sqrt()) * (-118348.0 / temperature).exp(),
        )
    }

    fn bremstrahlung_cooling_rate(&self) -> HeatingTerm {
        HeatingTerm::ergs_centimeters_cubed_per_s(1.42e-27 * self.temperature.in_kelvins().sqrt())
    }

    fn compton_cooling_rate(&self) -> EnergyPerTime {
        let x = (2.727 / self.scale_factor).value();
        EnergyPerTime::ergs_per_s(1.017e-37 * x.powi(4) * (self.temperature.in_kelvins() - x))
    }

    fn photoheating_rate(&self) -> HeatingRate {
        // TODO
        // let photon_average_energy: Energy = Energy::electron_volts(33.1);
        // let average_cross_section: CrossSection = todo!();
        // Rydberg
        // let average_energy = Energy::electron_volts(13.65693);
        // let energy_weighted_cross_section: CrossSection = todo!();
        // let heating_rate: HeatingRate = self.hydrogen_number_density()
        //     * photon_density
        //     * SPEED_OF_LIGHT
        //     * (SWEEP_HYDROGEN_ONLY_AVERAGE_PHOTON_ENERGY * energy_weighted_cross_section
        //         - average_energy * average_cross_section);
        let photon_density: NumberDensity = self.rate * self.length / SPEED_OF_LIGHT / self.volume;
        self.hydrogen_number_density()
            * photon_density
            * SPEED_OF_LIGHT
            * SWEEP_HYDROGEN_ONLY_AVERAGE_PHOTON_ENERGY
            * SWEEP_HYDROGEN_ONLY_CROSS_SECTION
    }

    fn cooling_rate(&self) -> HeatingRate {
        let ne = self.electron_number_density();
        let nh_neutral = self.hydrogen_number_density();
        let nh_ionized = self.ionized_hydrogen_number_density();
        let collisional = (self.collisional_excitation_cooling_rate()
            + self.collisional_ionization_cooling_rate())
            * ne
            * nh_neutral;
        let recombination = self.case_b_recombination_cooling_rate() * ne * nh_ionized;
        let bremsstrahlung = self.bremstrahlung_cooling_rate() * ne * nh_ionized;
        let compton: HeatingRate = self.compton_cooling_rate() * ne;
        collisional + recombination + bremsstrahlung + compton
    }

    fn temperature_change(&mut self, timestep: Time) -> Temperature {
        let rate = self.photoheating_rate() - self.cooling_rate();
        self.heating_rate = rate;
        let internal_energy_change = rate * timestep;
        Temperature::from_internal_energy_density_hydrogen_only(
            internal_energy_change,
            self.ionized_hydrogen_fraction,
            self.density,
        )
    }

    fn photoionization_rate(&self) -> Rate {
        let photon_density: NumberDensity = self.rate * self.length / SPEED_OF_LIGHT / self.volume;
        SWEEP_HYDROGEN_ONLY_CROSS_SECTION * SPEED_OF_LIGHT * photon_density
    }

    fn ionized_fraction_change(&self, timestep: Time) -> Dimensionless {
        // See A23 of Rosdahl et al
        let ne = self.electron_number_density();
        let nh = self.hydrogen_number_density();
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
        Ok(ideal_temperature_timestep.min(ideal_ionized_fraction_timestep))
    }

    fn perform_timestep_internal(
        &mut self,
        timestep: Time,
        timestep_safety_factor: Dimensionless,
        depth: usize,
    ) -> Time {
        let initial_state = (self.temperature, self.ionized_hydrogen_fraction);
        if depth > MAX_DEPTH {
            panic!(
                "Failed to find timestep in chemistry. Solver state: {:?}",
                self
            );
        }
        match self.try_timestep_update(timestep, timestep_safety_factor) {
            Err(TimestepCriterionViolated) => {
                (self.temperature, self.ionized_hydrogen_fraction) = initial_state;
                self.perform_timestep_internal(timestep / 2.0, timestep_safety_factor, depth + 1);
                self.perform_timestep_internal(timestep / 2.0, timestep_safety_factor, depth + 1)
            }
            Ok(timestep_recommendation) => timestep_recommendation,
        }
    }

    pub fn perform_timestep(
        &mut self,
        timestep: Time,
        timestep_safety_factor: Dimensionless,
    ) -> Time {
        self.perform_timestep_internal(timestep, timestep_safety_factor, 0)
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

    use diman::Quotient;

    use super::Solver;
    use crate::units::Density;
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
    use crate::units::VolumeRate;
    use crate::units::CASE_B_RECOMBINATION_RATE_HYDROGEN;
    use crate::units::PROTON_MASS;

    const MAX_ALLOWED_RELATIVE_CHANGE: f64 = 0.1;

    #[test]
    fn chemistry_solver_stays_in_equillibrium() {
        for initial_ionized_hydrogen_fraction in [
            Dimensionless::dimensionless(0.0),
            Dimensionless::dimensionless(0.2),
            Dimensionless::dimensionless(0.5),
            Dimensionless::dimensionless(0.7),
            Dimensionless::dimensionless(0.99),
            Dimensionless::dimensionless(1.0),
        ] {
            for timestep in [
                Time::megayears(1.0),
                Time::megayears(10.0),
                Time::megayears(100.0),
                Time::megayears(1000.0),
            ] {
                println!(
                    "Testing xHI = {initial_ionized_hydrogen_fraction:?}, Delta_t = {timestep:?}",
                );
                // Make sure this cell is optically thick by making it gigantic and dense
                let number_density = 1e5 / Volume::cubic_meters(1.0);
                let length = Length::kiloparsec(100.0);
                let volume = length.powi::<3>();
                // Set up rate such that recombination should be in equillibrium with ionization
                let recombination_rate = CASE_B_RECOMBINATION_RATE_HYDROGEN
                    * (number_density * initial_ionized_hydrogen_fraction).powi::<2>()
                    * volume;
                let rate = recombination_rate;
                let mut solver = Solver {
                    ionized_hydrogen_fraction: initial_ionized_hydrogen_fraction,
                    temperature: Temperature::kelvins(1000.0),
                    density: number_density * PROTON_MASS,
                    volume,
                    length,
                    rate,
                    scale_factor: Dimensionless::dimensionless(1.0),
                    heating_rate: HeatingRate::zero(),
                };
                solver.perform_timestep(
                    timestep,
                    Dimensionless::dimensionless(MAX_ALLOWED_RELATIVE_CHANGE),
                );
                let final_ionized_hydrogen_fraction = solver.ionized_hydrogen_fraction;
                assert!(
                    ((initial_ionized_hydrogen_fraction - final_ionized_hydrogen_fraction)
                        / (initial_ionized_hydrogen_fraction + 1e-20))
                        .value()
                        < 1e-10,
                );
            }
        }
    }

    fn run(mut solver: Solver, final_time: Time, timestep: Time, out: &str, output_cadence: usize) {
        let mut lines = vec![];
        let mut time = Time::zero();
        lines.push("t,xHI,T".into());
        for i in 0..(final_time / timestep).value() as usize {
            time += timestep;
            solver.perform_timestep(
                timestep,
                Dimensionless::dimensionless(MAX_ALLOWED_RELATIVE_CHANGE),
            );
            if i.rem_euclid(output_cadence) == 0 {
                lines.push(format!(
                    "{},{},{}",
                    time.in_megayears(),
                    solver.ionized_hydrogen_fraction.value(),
                    solver.temperature.in_kelvins()
                ));
            }
        }
        fs::write(out, lines.join("\n")).unwrap();
        dbg!(&solver);
    }

    #[test]
    fn time_evolution() {
        let timestep = Time::megayears(0.001);
        let length = Length::parsec(1.0);
        let volume = length.cubed();
        let final_time = Time::megayears(5000.0);
        // let flux = PhotonFlux::photons_per_s_per_cm_squared(1e5);
        let flux = PhotonFlux::photons_per_s_per_cm_squared(0.0);
        let area = volume / length;
        let rate = flux * area;

        for init_xhi in [0.0, 0.2, 0.5, 0.8, 1.0] {
            let ionized_hydrogen_fraction = Dimensionless::dimensionless(init_xhi);
            for temp_exp in [3, 4, 5, 6] {
                let temperature = Temperature::kelvins(10.0f64.powi(temp_exp));
                for exp in [-8, -6, -4, -2, 0, 2] {
                    let number_density = NumberDensity::per_centimeters_cubed(10.0f64.powi(exp));
                    let density = number_density * PROTON_MASS;

                    let solver = Solver {
                        ionized_hydrogen_fraction,
                        temperature,
                        density,
                        volume,
                        length,
                        rate,
                        scale_factor: Dimensionless::dimensionless(1.0),
                        heating_rate: HeatingRate::zero(),
                    };

                    let output = format!("out/{}_{}_{}", temp_exp, exp, init_xhi);
                    run(solver, final_time, timestep, &output, 1000);
                }
            }
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

    fn test_numerical_derivative(
        function: fn(&Solver) -> VolumeRate,
        derivative: fn(&Solver) -> Quotient<VolumeRate, Temperature>,
    ) {
        let epsilon = 1e-10;
        let delta = Temperature::kelvins(1e-10);
        for temperature in [
            Temperature::kelvins(1e1),
            Temperature::kelvins(1e2),
            Temperature::kelvins(1e3),
            Temperature::kelvins(1e4),
            Temperature::kelvins(1e5),
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
            dbg!(
                temperature,
                analytical,
                numerical,
                (analytical - numerical).abs() / (analytical.abs() + numerical.abs())
            );
            assert!(
                (analytical - numerical).abs()
                    / (analytical.abs() + numerical.abs() + Quantity::new_unchecked(f64::EPSILON))
                    < epsilon
            );
        }
    }
}
