use bevy::prelude::Resource;

use super::Chemistry;
use crate::grid::Cell;
use crate::sweep::site::Site;
use crate::units::Density;
use crate::units::Dimensionless;
use crate::units::EnergyPerTime;
use crate::units::HeatingRate;
use crate::units::Length;
use crate::units::PhotonFlux;
use crate::units::Rate;
use crate::units::Temperature;
use crate::units::Time;
use crate::units::Volume;
use crate::units::CASE_B_RECOMBINATION_RATE_HYDROGEN;
use crate::units::PROTON_MASS;

#[derive(Resource)]
pub struct HydrogenOnly {
    pub flux_treshold: PhotonFlux,
    pub scale_factor: Dimensionless,
}

#[derive(Debug)]
pub struct HydrogenOnlySpecies {
    pub ionized_hydrogen_fraction: Dimensionless,
    pub temperature: Temperature,
}

impl HydrogenOnlySpecies {
    pub(crate) fn new(
        ionized_hydrogen_fraction: Dimensionless,
        temperature: Temperature,
        density: Density,
    ) -> HydrogenOnlySpecies {
        Self {
            ionized_hydrogen_fraction,
            temperature,
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
        Solver {
            ionized_hydrogen_fraction: &mut site.species.ionized_hydrogen_fraction,
            temperature: &mut site.species.temperature,
            timestep,
            density: site.density,
            volume,
            length,
            flux,
            scale_factor: self.scale_factor,
        }
        .timestep();
        let relative_change =
            (old_fraction / (old_fraction - site.species.ionized_hydrogen_fraction)).abs();
        let change_timescale = relative_change * timestep;
        change_timescale
    }
}

pub(crate) struct Solver<'a> {
    pub ionized_hydrogen_fraction: &'a mut Dimensionless,
    pub temperature: &'a mut Temperature,
    pub timestep: Time,
    pub density: Density,
    pub volume: Volume,
    pub length: Length,
    pub flux: PhotonFlux,
    pub scale_factor: Dimensionless,
}

impl<'a> Solver<'a> {
    fn collision_fit_function(&self) -> f64 {
        let temperature = self.temperature.in_kelvins();
        temperature.sqrt() / (1.0 + (temperature / 1e5).sqrt()) * (-157809.1 / temperature).exp()
    }

    fn case_b_recombination_rate(&self) -> Rate {
        let lambda = Temperature::kelvins(315614.0) / *self.temperature;
        Rate::centimeters_cubed_per_s(
            2.753e-14 * lambda.powf(1.5) / (1.0 + (lambda / 2.74).powf(0.407)).powf(2.242),
        )
    }

    fn case_b_recombination_cooling_rate(&self) -> HeatingRate {
        let lambda = Temperature::kelvins(315614.0) / *self.temperature;
        HeatingRate::ergs_centimeters_cubed_per_s(
            3.435e-30 * self.temperature.in_kelvins() * lambda.powf(1.97)
                / (1.0 + (lambda / 2.25).powf(0.376)).powf(3.72),
        )
    }

    fn collisional_ionization_rate(&self) -> Rate {
        Rate::centimeters_cubed_per_s(5.85e-11 * self.collision_fit_function())
    }

    fn collisional_ionization_cooling_rate(&self) -> HeatingRate {
        HeatingRate::ergs_centimeters_cubed_per_s(1.27e-21 * self.collision_fit_function())
    }

    fn collisional_excitation_cooling_rate(&self) -> HeatingRate {
        let temperature = self.temperature.in_kelvins();
        HeatingRate::ergs_centimeters_cubed_per_s(
            7.5e-19 / (1.0 + (temperature / 1e5).sqrt()) * (-118348.0 / temperature).exp(),
        )
    }

    fn bremstrahlung_cooling_rate(&self) -> HeatingRate {
        HeatingRate::ergs_centimeters_cubed_per_s(1.42e-27 * self.temperature.in_kelvins().sqrt())
    }

    fn compton_cooling_rate(&self) -> EnergyPerTime {
        let x = (2.727 / self.scale_factor).value();
        EnergyPerTime::ergs_per_s(1.017e-37 * x.powi(4) * (self.temperature.in_kelvins() - x))
    }

    pub fn timestep(&mut self) {
        self.update_temperature();
        let temperature = self.temperature.in_kelvins();
        let hydrogen_number_density = self.density / PROTON_MASS;
        let num_hydrogen_atoms = hydrogen_number_density * self.volume;
        let recombination_rate = CASE_B_RECOMBINATION_RATE_HYDROGEN
            * (hydrogen_number_density * *self.ionized_hydrogen_fraction).powi::<2>()
            * self.volume;
        let num_recombined_hydrogen_atoms = (recombination_rate * self.timestep).to_amount();
        *self.ionized_hydrogen_fraction -=
            num_recombined_hydrogen_atoms / num_hydrogen_atoms.to_amount();
        let neutral_hydrogen_number_density =
            self.density / PROTON_MASS * (1.0 - *self.ionized_hydrogen_fraction);
        let sigma = crate::units::SWEEP_HYDROGEN_ONLY_CROSS_SECTION;
        let absorbed_fraction =
            1.0 - (-neutral_hydrogen_number_density * sigma * self.length).exp();
        let num_newly_ionized_hydrogen_atoms = (absorbed_fraction * self.flux) * self.timestep;
        *self.ionized_hydrogen_fraction +=
            num_newly_ionized_hydrogen_atoms / num_hydrogen_atoms.to_amount();
        *self.ionized_hydrogen_fraction = self.ionized_hydrogen_fraction.clamp(
            Dimensionless::dimensionless(0.0),
            Dimensionless::dimensionless(1.0),
        )
    }

    fn update_temperature(&mut self) {
        // let temperature = self.temperature.in_kelvins();
        // let photo_heating_rate = 1.0;
        // let heating = photo_heating_rate * self.flux;
        todo!()
        // let cooling = collisional_ionization + collisional_excitation + recombination + bremsstrahlung + compton;
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
                let mut final_ionized_hydrogen_fraction = initial_ionized_hydrogen_fraction;
                Solver {
                    ionized_hydrogen_fraction: &mut final_ionized_hydrogen_fraction,
                    temperature: &mut Temperature::kelvins(1000.0),
                    timestep,
                    density: number_density * PROTON_MASS,
                    volume,
                    length,
                    flux,
                    scale_factor: Dimensionless::dimensionless(1.0),
                }
                .timestep();
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
