use bevy::prelude::Resource;

use super::Chemistry;
use crate::sweep::grid::Cell;
use crate::sweep::site::Site;
use crate::units::Density;
use crate::units::Dimensionless;
use crate::units::Energy;
use crate::units::EnergyPerTime;
use crate::units::EnergyRateDensity;
use crate::units::HeatingRate;
use crate::units::HeatingTerm;
use crate::units::Length;
use crate::units::NumberDensity;
use crate::units::PhotonRate;
use crate::units::Rate;
use crate::units::Temperature;
use crate::units::Time;
use crate::units::Volume;
use crate::units::PROTON_MASS;
use crate::units::SPEED_OF_LIGHT;

#[derive(Resource)]
pub struct HydrogenOnly {
    pub rate_treshold: PhotonRate,
    pub scale_factor: Dimensionless,
}

#[derive(Debug)]
pub struct HydrogenOnlySpecies {
    pub ionized_hydrogen_fraction: Dimensionless,
    pub temperature: Temperature,
    pub heating_rate: HeatingRate,
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
        }
    }
}

impl Chemistry for HydrogenOnly {
    type Photons = PhotonRate;
    type Species = HydrogenOnlySpecies;

    fn get_outgoing_rate(
        &self,
        cell: &Cell,
        site: &mut Site<Self>,
        incoming_rate: Self::Photons,
    ) -> PhotonRate {
        let neutral_hydrogen_number_density =
            site.density / PROTON_MASS * (1.0 - site.species.ionized_hydrogen_fraction);
        let sigma = crate::units::SWEEP_HYDROGEN_ONLY_CROSS_SECTION;
        if incoming_rate < self.rate_treshold {
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
        let old_fraction = site.species.ionized_hydrogen_fraction;
        let mut solver = Solver {
            ionized_hydrogen_fraction: site.species.ionized_hydrogen_fraction,
            temperature: site.species.temperature,
            timestep,
            density: site.density,
            volume,
            length,
            rate,
            scale_factor: self.scale_factor,
        };
        let heating_rate = solver.timestep();
        site.species.temperature = solver.temperature;
        site.species.ionized_hydrogen_fraction = solver.ionized_hydrogen_fraction;
        site.species.heating_rate = heating_rate;
        let relative_change =
            (old_fraction / (old_fraction - site.species.ionized_hydrogen_fraction)).abs();
        let change_timescale = relative_change * timestep;
        change_timescale
    }
}

#[derive(Debug)]
pub(crate) struct Solver {
    pub ionized_hydrogen_fraction: Dimensionless,
    pub temperature: Temperature,
    pub timestep: Time,
    pub density: Density,
    pub volume: Volume,
    pub length: Length,
    pub rate: PhotonRate,
    pub scale_factor: Dimensionless,
}

// All numbers taken from Rosdahl et al (2015)
impl Solver {
    fn collision_fit_function(&self) -> f64 {
        let temperature = self.temperature.in_kelvins();
        temperature.sqrt() / (1.0 + (temperature / 1e5).sqrt()) * (-157809.1 / temperature).exp()
    }

    fn case_b_recombination_rate(&self) -> Rate {
        let lambda = Temperature::kelvins(315614.0) / self.temperature;
        Rate::centimeters_cubed_per_s(
            2.753e-14 * lambda.powf(1.5) / (1.0 + (lambda / 2.74).powf(0.407)).powf(2.242),
        )
    }

    fn case_b_recombination_cooling_rate(&self) -> HeatingTerm {
        let lambda = Temperature::kelvins(315614.0) / self.temperature;
        HeatingTerm::ergs_centimeters_cubed_per_s(
            3.435e-30 * self.temperature.in_kelvins() * lambda.powf(1.97)
                / (1.0 + (lambda / 2.25).powf(0.376)).powf(3.72),
        )
    }

    fn collisional_ionization_rate(&self) -> Rate {
        Rate::centimeters_cubed_per_s(5.85e-11 * self.collision_fit_function())
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

    fn photoheating(&self, absorbed_fraction: Dimensionless) -> Energy {
        let num_absorbed_photons: Dimensionless =
            self.rate * self.length / SPEED_OF_LIGHT * absorbed_fraction;
        let rydberg = Energy::electron_volts(13.65693);
        let average_energy: Energy = Energy::electron_volts(0.4298);
        let energy_per_photon = rydberg - average_energy;
        energy_per_photon * num_absorbed_photons
    }

    fn hydrogen_number_density(&self) -> NumberDensity {
        self.density / PROTON_MASS
    }

    fn neutral_hydrogen_number_density(&self) -> NumberDensity {
        self.density / PROTON_MASS * (1.0 - self.ionized_hydrogen_fraction)
    }

    fn ionized_hydrogen_number_density(&self) -> NumberDensity {
        self.density / PROTON_MASS * self.ionized_hydrogen_fraction
    }

    fn electron_number_density(&self) -> NumberDensity {
        // Assume zero ionized helium
        self.ionized_hydrogen_number_density()
    }

    fn get_heating_rate(&self, absorbed_fraction: Dimensionless) -> EnergyRateDensity {
        let photoheating = self.photoheating(absorbed_fraction) / self.volume / self.timestep;
        let ne = self.electron_number_density();
        let nh_neutral = self.hydrogen_number_density();
        let nh_ionized = self.ionized_hydrogen_number_density();
        let collisional = (self.collisional_excitation_cooling_rate()
            + self.collisional_ionization_cooling_rate())
            * ne
            * nh_neutral;
        let recombination = self.case_b_recombination_cooling_rate() * ne * nh_ionized;
        let bremsstrahlung = self.bremstrahlung_cooling_rate() * ne * nh_ionized;
        let compton: EnergyRateDensity = self.compton_cooling_rate() * ne;
        let cooling: EnergyRateDensity = collisional + recombination + bremsstrahlung + compton;
        photoheating - cooling
    }

    fn update_temperature(&mut self, absorbed_fraction: Dimensionless) -> HeatingRate {
        let rate = self.get_heating_rate(absorbed_fraction);
        let internal_energy_change = rate * self.timestep;
        let temperature_change = Temperature::from_internal_energy_density_hydrogen_only(
            internal_energy_change,
            self.ionized_hydrogen_fraction,
            self.density,
        );
        self.temperature += temperature_change;
        rate
    }

    pub fn timestep(&mut self) -> HeatingRate {
        let hydrogen_number_density = self.hydrogen_number_density();
        let recombination_rate = self.case_b_recombination_rate()
            * (hydrogen_number_density * self.ionized_hydrogen_fraction).powi::<2>();
        let recombined_density = recombination_rate * self.timestep;
        self.ionized_hydrogen_fraction -= recombined_density / hydrogen_number_density;
        let sigma = crate::units::SWEEP_HYDROGEN_ONLY_CROSS_SECTION;
        let absorbed_fraction =
            1.0 - (-self.neutral_hydrogen_number_density() * sigma * self.length).exp();
        let num_newly_ionized_hydrogen_atoms = (absorbed_fraction * self.rate) * self.timestep;
        let heating_rate = self.update_temperature(Dimensionless::dimensionless(absorbed_fraction));
        self.ionized_hydrogen_fraction +=
            num_newly_ionized_hydrogen_atoms / (hydrogen_number_density * self.volume);
        self.ionized_hydrogen_fraction = self.ionized_hydrogen_fraction.clamp(
            Dimensionless::dimensionless(0.0),
            Dimensionless::dimensionless(1.0),
        );
        heating_rate
    }
}

#[cfg(not(feature = "2d"))]
#[cfg(test)]
mod tests {
    use std::fs;

    use super::Solver;
    use crate::units::Dimensionless;
    use crate::units::Length;
    use crate::units::NumberDensity;
    use crate::units::PhotonFlux;
    use crate::units::Temperature;
    use crate::units::Time;
    use crate::units::Volume;
    use crate::units::CASE_B_RECOMBINATION_RATE_HYDROGEN;
    use crate::units::PROTON_MASS;

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
                    timestep,
                    density: number_density * PROTON_MASS,
                    volume,
                    length,
                    rate,
                    scale_factor: Dimensionless::dimensionless(1.0),
                };
                solver.timestep();
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
            solver.timestep();
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
                    let number_density =
                        NumberDensity::particles_per_centimeter_cubed(10.0f64.powi(exp));
                    let density = number_density * PROTON_MASS;

                    let solver = Solver {
                        ionized_hydrogen_fraction,
                        temperature,
                        timestep,
                        density,
                        volume,
                        length,
                        rate,
                        scale_factor: Dimensionless::dimensionless(1.0),
                    };

                    let output = format!("out/{}_{}_{}", temp_exp, exp, init_xhi);
                    run(solver, final_time, timestep, &output, 1000);
                }
            }
        }
    }
}
