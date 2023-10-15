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
use crate::components::Mass;
use crate::parameters::SimulationBox;
use crate::prelude::Particles;
use crate::units::Dimensionless;
use crate::units::PhotonRate;
use crate::units::Temperature;
use crate::units::Time;

#[derive(Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named, Serialize)]
#[name = "hydrogen_ionization_mass_average"]
pub struct HydrogenIonizationMassAverage(pub Dimensionless);

#[derive(Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named, Serialize)]
#[name = "hydrogen_ionization_volume_average"]
pub struct HydrogenIonizationVolumeAverage(pub Dimensionless);

#[derive(Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named, Serialize)]
#[name = "temperature_mass_average"]
pub struct TemperatureMassAverage(pub Temperature);

#[derive(Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named, Serialize)]
#[name = "photoionization_rate_volume_average"]
pub struct PhotoionizationRateVolumeAverage(pub PhotonRate);

#[derive(Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named, Serialize)]
#[name = "weighted_photoionization_rate_volume_average"]
pub struct WeightedPhotoionizationRateVolumeAverage(pub PhotonRate);

#[derive(Serialize, Clone, Named)]
#[name = "num_particles_at_timestep_levels"]
pub struct NumParticlesAtTimestepLevels(Vec<NumAtLevel>);

#[derive(Serialize, Clone)]
struct NumAtLevel {
    level: usize,
    num: usize,
    timestep: Time,
}

pub fn compute_time_series_system(
    mass_av_frac: Particles<(&components::Mass, &IonizedHydrogenFraction)>,
    volume_av_frac: Particles<(&Cell, &IonizedHydrogenFraction)>,
    mut mass_av_frac_writer: EventWriter<HydrogenIonizationMassAverage>,
    mut volume_av_frac_writer: EventWriter<HydrogenIonizationVolumeAverage>,
    temperature_mass_av: Particles<(&components::Temperature, &Mass)>,
    mut temperature_mass_av_writer: EventWriter<TemperatureMassAverage>,
    photoionization_rate: Particles<(&components::PhotoionizationRate, &Cell)>,
    mut photoionization_rate_writer: EventWriter<PhotoionizationRateVolumeAverage>,
    weighted_photoionization_rate: Particles<(
        &components::PhotoionizationRate,
        &IonizedHydrogenFraction,
        &Cell,
    )>,
    mut weighted_photoionization_rate_writer: EventWriter<WeightedPhotoionizationRateVolumeAverage>,
    box_: Res<SimulationBox>,
) {
    let ionized_mass = compute_global_sum(mass_av_frac.iter().map(|(mass, frac)| **mass * **frac));
    let total_mass = compute_global_sum(mass_av_frac.iter().map(|(mass, _)| **mass));
    let ratio = ionized_mass / total_mass;
    debug!(
        "{:<41}: {:.2}%",
        "Mass av. ionized hydrogen fraction",
        ratio.in_percent()
    );
    mass_av_frac_writer.send(HydrogenIonizationMassAverage(ratio));

    let ionized_volume = compute_global_sum(
        volume_av_frac
            .iter()
            .map(|(cell, frac)| cell.volume() * **frac),
    );
    let total_volume = compute_global_sum(volume_av_frac.iter().map(|(cell, _)| cell.volume()));
    let ratio = ionized_volume / total_volume;
    debug!(
        "{:<41}: {:.2}%",
        "Volume av. ionized hydrogen fraction",
        ratio.in_percent()
    );
    volume_av_frac_writer.send(HydrogenIonizationVolumeAverage(ratio));

    let mass_weighted_temperature = compute_global_sum(
        temperature_mass_av
            .iter()
            .map(|(temp, mass)| **temp * **mass),
    );
    let total_mass = compute_global_sum(temperature_mass_av.iter().map(|(_, mass)| **mass));
    let average = mass_weighted_temperature / total_mass;
    debug!(
        "{:<41}: {:.5} K",
        "Mass av. temperature",
        average.in_kelvins()
    );
    temperature_mass_av_writer.send(TemperatureMassAverage(average));

    let volume_weighted_rate = compute_global_sum(
        photoionization_rate
            .iter()
            .map(|(rate, cell)| **rate * cell.volume()),
    );
    let total_volume =
        compute_global_sum(photoionization_rate.iter().map(|(_, cell)| cell.volume()));
    let average = volume_weighted_rate / total_volume * box_.volume();
    debug!(
        "{:<41}: {:.5e} s^-1",
        "Volume av. photoionization rate",
        average.in_photons_per_second()
    );
    photoionization_rate_writer.send(PhotoionizationRateVolumeAverage(average));

    let volume_weighted_rate = compute_global_sum(
        weighted_photoionization_rate
            .iter()
            .map(|(rate, ion_frac, cell)| **rate * **ion_frac * cell.volume()),
    );
    let total_volume =
        compute_global_sum(photoionization_rate.iter().map(|(_, cell)| cell.volume()));
    let average = volume_weighted_rate / total_volume * box_.volume();
    debug!(
        "{:<41}: {:.5e} s^-1",
        "Volume av. weighted photoionization rate",
        average.in_photons_per_second()
    );
    weighted_photoionization_rate_writer.send(WeightedPhotoionizationRateVolumeAverage(average));
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
