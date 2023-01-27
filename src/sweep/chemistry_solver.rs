use crate::units::Density;
use crate::units::Dimensionless;
use crate::units::Length;
use crate::units::PhotonFlux;
use crate::units::SourceRate;
use crate::units::Time;
use crate::units::Volume;
use crate::units::CASE_B_RECOMBINATION_RATE_HYDROGEN;
use crate::units::PROTON_MASS;

pub fn solve_chemistry(
    ionized_hydrogen_fraction: &mut Dimensionless,
    timestep: Time,
    density: Density,
    volume: Volume,
    size: Length,
    source: SourceRate,
    flux: PhotonFlux,
) {
    let hydrogen_number_density = density / PROTON_MASS;
    let num_hydrogen_atoms = hydrogen_number_density * volume;
    let recombination_rate = CASE_B_RECOMBINATION_RATE_HYDROGEN
        * (hydrogen_number_density * *ionized_hydrogen_fraction).powi::<2>();
    let num_recombined_hydrogen_atoms = (recombination_rate * timestep * volume).to_amount();
    let neutral_hydrogen_number_density =
        density / PROTON_MASS * (1.0 - *ionized_hydrogen_fraction);
    let sigma = crate::units::SWEEP_HYDROGEN_ONLY_CROSS_SECTION;
    let flux = flux + source;
    let absorbed_fraction = 1.0 - (-neutral_hydrogen_number_density * sigma * size).exp();
    let num_newly_ionized_hydrogen_atoms = (absorbed_fraction * flux) * timestep;
    *ionized_hydrogen_fraction += (num_newly_ionized_hydrogen_atoms
        - num_recombined_hydrogen_atoms)
        / num_hydrogen_atoms.to_amount();
    *ionized_hydrogen_fraction = ionized_hydrogen_fraction.clamp(
        Dimensionless::dimensionless(0.0),
        Dimensionless::dimensionless(1.0),
    );
}
