use std::iter::Sum;
use std::ops::Div;

use bevy::prelude::*;
use derive_custom::Named;
use derive_more::From;
use hdf5::H5Type;
use mpi::traits::Equivalence;

use super::grid::Cell;
use crate::communication::communicator::Communicator;
use crate::components::Density;
use crate::components::IonizedHydrogenFraction;
use crate::prelude::Particles;
use crate::units::Dimensionless;
use crate::units::Mass;
use crate::units::Volume;

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named)]
#[name = "hydrogen_ionization_mass_average"]
#[repr(transparent)]
pub struct HydrogenIonizationMassAverage(pub Dimensionless);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named)]
#[name = "hydrogen_ionization_volume_average"]
#[repr(transparent)]
pub struct HydrogenIonizationVolumeAverage(pub Dimensionless);

pub fn hydrogen_ionization_volume_average_system(
    query: Particles<(&Cell, &Density, &IonizedHydrogenFraction)>,
    mut writer: EventWriter<HydrogenIonizationVolumeAverage>,
) {
    let average = compute_global_average::<Volume>(
        query,
        |(cell, _, frac)| **frac * cell.volume(),
        |(cell, _, _)| cell.volume(),
    );
    writer.send(HydrogenIonizationVolumeAverage(average));
}

pub fn hydrogen_ionization_mass_average_system(
    query: Particles<(&Cell, &Density, &IonizedHydrogenFraction)>,
    mut writer: EventWriter<HydrogenIonizationMassAverage>,
) {
    let average = compute_global_average::<Mass>(
        query,
        |(cell, density, frac)| **frac * cell.volume() * **density,
        |(cell, density, _)| cell.volume() * **density,
    );
    writer.send(HydrogenIonizationMassAverage(average));
}

fn compute_global_average<T>(
    query: Particles<(&Cell, &Density, &IonizedHydrogenFraction)>,
    fn_1: impl Fn((&Cell, &Density, &IonizedHydrogenFraction)) -> T,
    fn_2: impl Fn((&Cell, &Density, &IonizedHydrogenFraction)) -> T,
) -> Dimensionless
where
    T: Equivalence + Sum<T> + Clone + Div<T, Output = Dimensionless> + 'static,
{
    let mut comm: Communicator<T> = Communicator::new();
    let ionized: T = query.iter().map(fn_1).sum();
    let total: T = query.iter().map(fn_2).sum();
    let ionized: T = comm.all_gather_sum(&ionized);
    let total: T = comm.all_gather_sum(&total);
    ionized / total
}
