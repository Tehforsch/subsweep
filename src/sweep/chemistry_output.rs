use bevy_ecs::prelude::*;

use super::Sweep;
use crate::chemistry::hydrogen_only::HydrogenOnly;
use crate::chemistry::hydrogen_only::Solver;
use crate::components::CollisionalIonizationRate;
use crate::components::HeatingRate;
use crate::components::PhotoionizationRate;
use crate::components::RecombinationRate;
use crate::parameters::Cosmology;
use crate::prelude::ParticleId;
use crate::prelude::Particles;

pub trait ChemistryOutputType {
    fn from_solver(solver: &Solver) -> Self;
}

impl ChemistryOutputType for PhotoionizationRate {
    fn from_solver(solver: &Solver) -> Self {
        PhotoionizationRate(solver.photoionization_rate())
    }
}

impl ChemistryOutputType for HeatingRate {
    fn from_solver(solver: &Solver) -> Self {
        HeatingRate(solver.photoheating_rate() - solver.cooling_rate())
    }
}

impl ChemistryOutputType for RecombinationRate {
    fn from_solver(solver: &Solver) -> Self {
        RecombinationRate(
            solver.case_b_recombination_rate()
                * solver.electron_number_density()
                * solver.ionized_hydrogen_fraction,
        )
    }
}

impl ChemistryOutputType for CollisionalIonizationRate {
    fn from_solver(solver: &Solver) -> Self {
        CollisionalIonizationRate(
            solver.collisional_ionization_rate()
                * solver.electron_number_density()
                * (1.0 - solver.ionized_hydrogen_fraction),
        )
    }
}

pub fn sweep_optional_output_system<C: ChemistryOutputType + Component>(
    mut solver: NonSendMut<Option<Sweep<HydrogenOnly>>>,
    mut items: Particles<(&ParticleId, &mut C)>,
    cosmology: Res<Cosmology>,
) {
    let solver = (*solver).as_mut().unwrap();
    for (id, mut item) in items.iter_mut() {
        let solver = solver.get_solver(*id, cosmology.scale_factor());
        *item = C::from_solver(&solver);
    }
}
