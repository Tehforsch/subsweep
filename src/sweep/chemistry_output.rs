use bevy_ecs::prelude::*;

use super::Sweep;
use crate::chemistry::hydrogen_only::HydrogenOnly;
use crate::chemistry::hydrogen_only::Solver;
use crate::components::HeatingRate;
use crate::components::PhotoionizationRate;
use crate::parameters::Cosmology;
use crate::prelude::ParticleId;
use crate::prelude::Particles;
use crate::units::Time;

const TIMESTEP_YRS: f64 = 1.0;

fn timestep() -> Time {
    Time::years(TIMESTEP_YRS)
}

pub trait ChemistryOutputType {
    fn from_solver(solver: &Solver) -> Self;
}

impl ChemistryOutputType for PhotoionizationRate {
    fn from_solver(solver: &Solver) -> Self {
        PhotoionizationRate(solver.photoionization_rate(timestep()) / solver.volume)
    }
}

impl ChemistryOutputType for HeatingRate {
    fn from_solver(solver: &Solver) -> Self {
        HeatingRate(solver.photoheating_rate(timestep()) - solver.cooling_rate())
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
