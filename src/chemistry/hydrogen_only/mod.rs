use bevy::prelude::Resource;
use diman::Quotient;

use super::Chemistry;
use crate::grid::Cell;
use crate::sweep::site::Site;
use crate::units::CrossSection;
use crate::units::Density;
use crate::units::Dimensionless;
use crate::units::Energy;
use crate::units::EnergyPerTime;
use crate::units::HeatingRate;
use crate::units::HeatingTerm;
use crate::units::Length;
use crate::units::NumberDensity;
use crate::units::PhotonFlux;
use crate::units::Rate;
use crate::units::Temperature;
use crate::units::Time;
use crate::units::Volume;
use crate::units::CASE_B_RECOMBINATION_RATE_HYDROGEN;
use crate::units::PROTON_MASS;
use crate::units::SPEED_OF_LIGHT;

#[derive(Resource)]
pub struct HydrogenOnly {
    pub flux_treshold: PhotonFlux,
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
    type Photons = PhotonFlux;
    type Species = HydrogenOnlySpecies;

    fn get_outgoing_flux(
        &self,
        cell: &Cell,
        site: &mut Site<Self>,
        incoming_flux: Self::Photons,
    ) -> PhotonFlux {
        let neutral_hydrogen_number_density =
            site.density / PROTON_MASS * (1.0 - site.species.ionized_hydrogen_fraction);
        let sigma = crate::units::SWEEP_HYDROGEN_ONLY_CROSS_SECTION;
        if incoming_flux < self.flux_treshold {
            PhotonFlux::zero()
        } else {
            let absorbed_fraction = (-neutral_hydrogen_number_density * sigma * cell.size).exp();
            incoming_flux * absorbed_fraction
        }
    }

    fn update_abundances(
        &self,
        site: &mut Site<Self>,
        flux: Self::Photons,
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
            flux,
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

pub(crate) struct Solver {
    pub ionized_hydrogen_fraction: Dimensionless,
    pub temperature: Temperature,
    pub timestep: Time,
    pub density: Density,
    pub volume: Volume,
    pub length: Length,
    pub flux: PhotonFlux,
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

    fn photoheating(&self) -> Quotient<Energy, Time> {
        let rydberg = Energy::electron_volts(13.65693);
        let average_energy: Energy = Energy::electron_volts(0.4298);
        let average_cross_section: CrossSection = CrossSection::centimeters_squared(5.475e-14);
        let photon_density = self.flux * self.length / SPEED_OF_LIGHT;
        self.neutral_hydrogen_number_density()
            * SPEED_OF_LIGHT
            * photon_density.remove_amount()
            * (rydberg - average_energy)
            * average_cross_section
    }

    fn hydrogen_number_density(&self) -> NumberDensity {
        self.density / PROTON_MASS
    }

    fn neutral_hydrogen_number_density(&self) -> NumberDensity {
        self.density / PROTON_MASS * self.ionized_hydrogen_fraction
    }

    fn ionized_hydrogen_number_density(&self) -> NumberDensity {
        self.density / PROTON_MASS * self.ionized_hydrogen_fraction
    }

    fn electron_number_density(&self) -> NumberDensity {
        // Assume zero ionized helium
        self.ionized_hydrogen_number_density()
    }

    fn get_heating_rate(&self) -> EnergyPerTime {
        let photoheating = self.photoheating();
        let ne = self.electron_number_density();
        let nh_neutral = self.hydrogen_number_density();
        let nh_ionized = self.ionized_hydrogen_number_density();
        let collisional = (self.collisional_excitation_cooling_rate()
            + self.collisional_ionization_cooling_rate())
            * ne
            * nh_neutral;
        let recombination = self.case_b_recombination_cooling_rate() * ne * nh_ionized;
        let bremsstrahlung = self.bremstrahlung_cooling_rate() * ne * nh_ionized;
        let compton = self.compton_cooling_rate() * ne;
        photoheating
            - self.volume
                * ((collisional + recombination + bremsstrahlung).remove_amount() + compton)
    }

    fn update_temperature(&mut self) -> HeatingRate {
        let rate = self.get_heating_rate();
        let internal_energy_change = rate * self.timestep / self.volume;
        let temperature_change = Temperature::from_internal_energy_density_hydrogen_only(
            internal_energy_change,
            self.ionized_hydrogen_fraction,
            self.density,
        );
        if temperature_change > Temperature::kelvins(1e0) {
            dbg!(rate, internal_energy_change, temperature_change);
        }
        self.temperature += temperature_change;
        rate / self.volume
    }

    pub fn timestep(&mut self) -> HeatingRate {
        let heating_rate = self.update_temperature();
        let hydrogen_number_density = self.hydrogen_number_density();
        let num_hydrogen_atoms = hydrogen_number_density * self.volume;
        let recombination_rate = CASE_B_RECOMBINATION_RATE_HYDROGEN
            * (hydrogen_number_density * self.ionized_hydrogen_fraction).powi::<2>()
            * self.volume;
        let num_recombined_hydrogen_atoms = (recombination_rate * self.timestep).to_amount();
        self.ionized_hydrogen_fraction -=
            num_recombined_hydrogen_atoms / num_hydrogen_atoms.to_amount();
        let neutral_hydrogen_number_density =
            self.density / PROTON_MASS * (1.0 - self.ionized_hydrogen_fraction);
        let sigma = crate::units::SWEEP_HYDROGEN_ONLY_CROSS_SECTION;
        let absorbed_fraction =
            1.0 - (-neutral_hydrogen_number_density * sigma * self.length).exp();
        let num_newly_ionized_hydrogen_atoms = (absorbed_fraction * self.flux) * self.timestep;
        self.ionized_hydrogen_fraction +=
            num_newly_ionized_hydrogen_atoms / num_hydrogen_atoms.to_amount();
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
    use super::Solver;
    use crate::units::Amount;
    use crate::units::Dimensionless;
    use crate::units::Length;
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
                // Set up flux such that recombination should be in equillibrium with ionization
                let recombination_rate = CASE_B_RECOMBINATION_RATE_HYDROGEN
                    * (number_density * initial_ionized_hydrogen_fraction).powi::<2>()
                    * volume;
                let flux = recombination_rate * Amount::one_unchecked();
                let mut solver = Solver {
                    ionized_hydrogen_fraction: initial_ionized_hydrogen_fraction,
                    temperature: Temperature::kelvins(1000.0),
                    timestep,
                    density: number_density * PROTON_MASS,
                    volume,
                    length,
                    flux,
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
}
