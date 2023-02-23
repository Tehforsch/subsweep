#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use std::f64::consts::PI;

use bevy::prelude::*;
use derive_more::From;
use derive_more::Into;
use hdf5::H5Type;
use mpi::traits::Equivalence;
use mpi::Rank;
use ordered_float::OrderedFloat;
use raxiom::components;
use raxiom::components::IonizedHydrogenFraction;
use raxiom::components::Position;
use raxiom::grid::init_cartesian_grid_system;
use raxiom::grid::Cell;
use raxiom::grid::NumCellsSpec;
use raxiom::hydrodynamics::HaloParticle;
use raxiom::io::time_series::TimeSeriesPlugin;
use raxiom::prelude::*;
use raxiom::simulation_plugin::time_system;
use raxiom::sweep::timestep_level::TimestepLevel;
use raxiom::sweep::SweepParameters;
use raxiom::units::Amount;
use raxiom::units::Dimensionless;
use raxiom::units::Length;
use raxiom::units::NumberDensity;
use raxiom::units::PhotonFlux;
use raxiom::units::VecLength;
use raxiom::units::Volume;
use raxiom::units::CASE_B_RECOMBINATION_RATE_HYDROGEN;
use raxiom::units::PROTON_MASS;

#[derive(Named, Debug, H5Type, Clone, Deref, From)]
#[name = "rtype_error"]
#[repr(transparent)]
struct RTypeError(Dimensionless);

#[derive(Named, Debug, H5Type, Clone, Deref, From)]
#[name = "rtype_radius"]
#[repr(transparent)]
struct RTypeRadius(Length);

#[derive(Equivalence, Clone, Into)]
#[repr(transparent)]
struct RTypeVolume(Volume);

#[derive(Debug, Equivalence, Clone, PartialOrd, PartialEq)]
struct DistanceToSourceData {
    distance: Length,
    rank: Rank,
}

#[raxiom_parameters("rtype")]
struct Parameters {
    resolution: NumCellsSpec,
    number_density: NumberDensity,
    initial_fraction_ionized_hydrogen: Dimensionless,
    source_strength: PhotonFlux,
    source_pos: VecLength,
}

fn main() {
    let mut sim = SimulationBuilder::new();
    let mut sim = sim
        .parameters_from_relative_path(file!(), "parameters.yml")
        .headless(false)
        .write_output(true)
        .read_initial_conditions(false)
        .update_from_command_line_options()
        .build();
    let parameters = sim
        .add_parameter_type_and_get_result::<Parameters>()
        .clone();
    sim.add_startup_system(
        move |commands: Commands,
              box_size: Res<SimulationBox>,
              world_size: Res<WorldSize>,
              world_rank: Res<WorldRank>| {
            init_cartesian_grid_system(
                commands,
                box_size,
                parameters.resolution,
                world_size,
                world_rank,
            )
        },
    )
    .add_startup_system_to_stage(
        SimulationStartupStages::InsertDerivedComponents,
        initialize_sweep_components_system,
    )
    .add_startup_system_to_stage(
        SimulationStartupStages::InsertDerivedComponents,
        initialize_source_system,
    )
    .add_system_to_stage(
        SimulationStages::Integration,
        print_ionization_system.after(time_system),
    )
    .add_plugin(CommunicationPlugin::<RTypeVolume>::default())
    .add_plugin(CommunicationPlugin::<DistanceToSourceData>::default())
    .add_plugin(TimeSeriesPlugin::<RTypeRadius>::default())
    .add_plugin(TimeSeriesPlugin::<RTypeError>::default())
    .add_plugin(SweepPlugin)
    .run();
}

fn initialize_source_system(
    mut commands: Commands,
    particles: Particles<(Entity, &Position)>,
    parameters: Res<Parameters>,
    mut comm: Communicator<DistanceToSourceData>,
    world_rank: Res<WorldRank>,
) {
    let (closest_entity_to_pos, distance) = particles
        .iter()
        .map(|(entity, pos)| {
            let dist = **pos - parameters.source_pos;
            (entity, OrderedFloat(dist.length().value_unchecked()))
        })
        .min_by_key(|(_, dist)| *dist)
        .unwrap();
    let rank_with_min_distance: Rank = comm
        .all_gather_min::<DistanceToSourceData>(&DistanceToSourceData {
            distance: Length::new_unchecked(*distance),
            rank: **world_rank,
        })
        .unwrap()
        .rank;
    if **world_rank == rank_with_min_distance {
        commands
            .entity(closest_entity_to_pos)
            .insert(components::Source(parameters.source_strength));
    }
}

fn initialize_sweep_components_system(
    mut commands: Commands,
    local_particles: Query<Entity, With<LocalParticle>>,
    halo_particles: Query<Entity, With<HaloParticle>>,
    sweep_parameters: Res<SweepParameters>,
    parameters: Res<Parameters>,
) {
    for entity in local_particles.iter() {
        commands.entity(entity).insert((
            components::Density(parameters.number_density * PROTON_MASS),
            components::IonizedHydrogenFraction(parameters.initial_fraction_ionized_hydrogen),
            TimestepLevel(sweep_parameters.num_timestep_levels - 1),
        ));
    }
    for entity in halo_particles.iter() {
        commands
            .entity(entity)
            .insert((TimestepLevel(sweep_parameters.num_timestep_levels - 1),));
    }
}

fn print_ionization_system(
    ionization: Query<(&Cell, &IonizedHydrogenFraction)>,
    time: Res<raxiom::simulation_plugin::Time>,
    mut radius_writer: EventWriter<RTypeRadius>,
    mut error_writer: EventWriter<RTypeError>,
    mut comm: Communicator<RTypeVolume>,
    parameters: Res<Parameters>,
) {
    let mut volume = Volume::zero();
    for (cell, frac) in ionization.iter() {
        volume += **frac * cell.volume();
    }
    let volume: Volume = comm.all_gather_sum(&RTypeVolume(volume));
    let rate = parameters.source_strength / Amount::one_unchecked();
    let stroemgren_radius = (3.0 * rate
        / (4.0 * PI * CASE_B_RECOMBINATION_RATE_HYDROGEN * parameters.number_density.powi::<2>()))
    .cbrt();
    let recombination_time =
        (CASE_B_RECOMBINATION_RATE_HYDROGEN * parameters.number_density).powi::<-1>();
    let radius: Length = (volume / (4.0 * PI / 3.0)).cbrt();
    let analytical = (1.0 - (-**time / recombination_time).exp()).cbrt() * stroemgren_radius;
    let error = (radius - analytical).abs() / (radius.max(analytical));
    info!(
        "Radius of ionized region: {:.3?} (analytical: {:.3?}, {:.3?}%)",
        radius,
        analytical,
        error.in_percent(),
    );
    radius_writer.send(RTypeRadius(radius));
    error_writer.send(RTypeError(error));
}
