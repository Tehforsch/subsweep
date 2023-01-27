use crate::units::Density;
use crate::units::Dimensionless;
use crate::units::Length;
use crate::units::PhotonFlux;
use crate::units::Time;
use crate::units::Volume;
use crate::units::CASE_B_RECOMBINATION_RATE_HYDROGEN;
use crate::units::PROTON_MASS;

pub struct Solver {
    pub ionized_hydrogen_fraction: Dimensionless,
    pub timestep: Time,
    pub density: Density,
    pub volume: Volume,
    pub length: Length,
    pub flux: PhotonFlux,
}

impl Solver {
    pub fn get_new_abundance(mut self) -> Dimensionless {
        let hydrogen_number_density = self.density / PROTON_MASS;
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
        self.ionized_hydrogen_fraction.clamp(
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
                let final_ionized_hydrogen_fraction = Solver {
                    ionized_hydrogen_fraction: initial_ionized_hydrogen_fraction,
                    timestep,
                    density: number_density * PROTON_MASS,
                    volume,
                    length,
                    flux,
                }
                .get_new_abundance();
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
