use std::iter::Sum;
use std::ops::Div;

use bevy_ecs::prelude::*;
use derive_custom::Named;
use derive_more::Deref;
use derive_more::DerefMut;
use derive_more::From;
use log::debug;
use mpi::traits::Equivalence;
use serde::Serialize;

use super::grid::Cell;
use crate::communication::communicator::Communicator;
use crate::components::Density;
use crate::components::IonizedHydrogenFraction;
use crate::prelude::Particles;
use crate::units::Dimensionless;
use crate::units::Mass;
use crate::units::Volume;

#[derive(Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named, Serialize)]
#[name = "hydrogen_ionization_mass_average"]
#[repr(transparent)]
pub struct HydrogenIonizationMassAverage(pub Dimensionless);

#[derive(Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named, Serialize)]
#[name = "hydrogen_ionization_volume_average"]
#[repr(transparent)]
pub struct HydrogenIonizationVolumeAverage(pub Dimensionless);

pub fn hydrogen_ionization_volume_average_system(
    query: Particles<(&Cell, &Density, &IonizedHydrogenFraction)>,
    mut writer: EventWriter<HydrogenIonizationVolumeAverage>,
) {
    let comm = Communicator::new_custom_tag(10000);
    let average = compute_global_average::<Volume>(
        comm,
        query,
        |(cell, _, frac)| **frac * cell.volume(),
        |(cell, _, _)| cell.volume(),
    );
    debug!(
        "Volume av. ionized hydrogen fraction: {:.2}%",
        average.in_percent()
    );
    writer.send(HydrogenIonizationVolumeAverage(average));
}

pub fn hydrogen_ionization_mass_average_system(
    query: Particles<(&Cell, &Density, &IonizedHydrogenFraction)>,
    mut writer: EventWriter<HydrogenIonizationMassAverage>,
) {
    let comm = Communicator::new_custom_tag(10001);
    let average = compute_global_average::<Mass>(
        comm,
        query,
        |(cell, density, frac)| **frac * cell.volume() * **density,
        |(cell, density, _)| cell.volume() * **density,
    );
    debug!(
        "Mass av. ionized hydrogen fraction: {:.2}%",
        average.in_percent()
    );
    writer.send(HydrogenIonizationMassAverage(average));
}

fn compute_global_average<T>(
    mut comm: Communicator<T>,
    query: Particles<(&Cell, &Density, &IonizedHydrogenFraction)>,
    fn_1: impl Fn((&Cell, &Density, &IonizedHydrogenFraction)) -> T,
    fn_2: impl Fn((&Cell, &Density, &IonizedHydrogenFraction)) -> T,
) -> Dimensionless
where
    T: Equivalence + Sum<T> + Clone + Div<T, Output = Dimensionless> + 'static,
{
    let ionized: T = query.iter().map(fn_1).sum();
    let total: T = query.iter().map(fn_2).sum();
    let ionized: T = comm.all_gather_sum(&ionized);
    let total: T = comm.all_gather_sum(&total);
    ionized / total
}
