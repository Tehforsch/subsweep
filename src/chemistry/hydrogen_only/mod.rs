use bevy::prelude::Resource;

use super::Chemistry;
use crate::grid::Cell;
use crate::sweep::site::Site;
use crate::units::Density;
use crate::units::Dimensionless;
use crate::units::EnergyDensity;
use crate::units::Length;
use crate::units::PhotonFlux;
use crate::units::Temperature;
use crate::units::Time;
use crate::units::Volume;
use crate::units::CASE_B_RECOMBINATION_RATE_HYDROGEN;
use crate::units::PROTON_MASS;

#[derive(Resource)]
pub struct HydrogenOnly {
    pub flux_treshold: PhotonFlux,
}

#[derive(Debug)]
pub struct HydrogenOnlySpecies {
    pub ionized_hydrogen_fraction: Dimensionless,
    pub internal_energy_density: EnergyDensity,
}

impl HydrogenOnlySpecies {
    pub(crate) fn new(
        ionized_hydrogen_fraction: Dimensionless,
        temperature: Temperature,
        density: Density,
    ) -> HydrogenOnlySpecies {
        let internal_energy_density = EnergyDensity::from_temperature_hydrogen_only(
            temperature,
            ionized_hydrogen_fraction,
            density,
        );
        Self {
            ionized_hydrogen_fraction,
            internal_energy_density,
        }
    }
}

impl Site<HydrogenOnly> {
    pub(crate) fn get_temperature(&self, density: Density) -> Temperature {
        Temperature::from_internal_energy_density_hydrogen_only(
            self.species.internal_energy_density,
            self.species.ionized_hydrogen_fraction,
            density,
        )
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
            internal_energy_density: &mut site.species.internal_energy_density,
            timestep,
            density: site.density,
            volume,
            length,
            flux,
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
    pub internal_energy_density: &'a mut EnergyDensity,
    pub timestep: Time,
    pub density: Density,
    pub volume: Volume,
    pub length: Length,
    pub flux: PhotonFlux,
}

impl<'a> Solver<'a> {
    pub fn timestep(&mut self) {
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
}

#[cfg(not(feature = "2d"))]
#[cfg(test)]
mod tests {
    use super::Solver;
    use crate::units::Amount;
    use crate::units::Dimensionless;
    use crate::units::EnergyDensity;
    use crate::units::Length;
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
                    internal_energy_density: &mut EnergyDensity::zero(),
                    timestep,
                    density: number_density * PROTON_MASS,
                    volume,
                    length,
                    flux,
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
