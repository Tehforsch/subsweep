use std::iter;

use bevy_ecs::prelude::*;
use derive_custom::Named;
use derive_more::Deref;
use derive_more::DerefMut;
use derive_more::From;
use log::debug;
use mpi::traits::Equivalence;
use serde::Serialize;

use super::grid::Cell;
use super::Sweep;
use super::SweepParameters;
use crate::chemistry::Chemistry;
use crate::communication::communicator::Communicator;
use crate::components;
use crate::components::IonizedHydrogenFraction;
use crate::prelude::Particles;
use crate::units::Dimensionless;
use crate::units::Time;

#[derive(Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named, Serialize)]
#[name = "hydrogen_ionization_mass_average"]
pub struct HydrogenIonizationMassAverage(pub Dimensionless);

#[derive(Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named, Serialize)]
#[name = "hydrogen_ionization_volume_average"]
pub struct HydrogenIonizationVolumeAverage(pub Dimensionless);

#[derive(Serialize, Clone, Named)]
#[name = "num_particles_at_timestep_levels"]
pub struct NumParticlesAtTimestepLevels(Vec<NumAtLevel>);

#[derive(Serialize, Clone)]
struct NumAtLevel {
    level: usize,
    num: usize,
    timestep: Time,
}

pub fn hydrogen_ionization_mass_average_system(
    query: Particles<(&components::Mass, &IonizedHydrogenFraction)>,
    mut writer: EventWriter<HydrogenIonizationMassAverage>,
) {
    let ionized_mass = compute_global_sum(query.iter().map(|(mass, frac)| **mass * **frac));
    let total_mass = compute_global_sum(query.iter().map(|(mass, _)| **mass));
    let ratio = ionized_mass / total_mass;
    debug!(
        "Mass av. ionized hydrogen fraction: {:.2}%",
        ratio.in_percent()
    );
    writer.send(HydrogenIonizationMassAverage(ratio));
}

pub fn hydrogen_ionization_volume_average_system(
    query: Particles<(&Cell, &IonizedHydrogenFraction)>,
    mut writer: EventWriter<HydrogenIonizationMassAverage>,
) {
    let ionized_volume =
        compute_global_sum(query.iter().map(|(cell, frac)| cell.volume() * **frac));
    let total_volume = compute_global_sum(query.iter().map(|(cell, _)| cell.volume()));
    let ratio = ionized_volume / total_volume;
    debug!(
        "Volume av. ionized hydrogen fraction: {:.2}%",
        ratio.in_percent()
    );
    writer.send(HydrogenIonizationMassAverage(ratio));
}

fn compute_global_sum<T>(i: impl Iterator<Item = T>) -> T
where
    T: iter::Sum<T> + Clone + Equivalence + 'static,
{
    let mut comm = Communicator::new();
    let local_value: T = i.sum();
    let value: T = comm.all_gather_sum(&local_value);
    value
}

pub(super) fn num_particles_at_timestep_levels_system<C: Chemistry>(
    mut solver: NonSendMut<Option<Sweep<C>>>,
    mut writer: EventWriter<NumParticlesAtTimestepLevels>,
    parameters: Res<SweepParameters>,
) {
    let solver = (*solver).as_mut().unwrap();
    let max_timestep = parameters.max_timestep;
    writer.send(NumParticlesAtTimestepLevels(
        solver
            .timestep_state
            .iter_all_levels()
            .map(|level| {
                let num = solver.count_cells_global(level);
                NumAtLevel {
                    level: level.0,
                    num,
                    timestep: level.to_timestep(max_timestep),
                }
            })
            .collect(),
    ));
}
