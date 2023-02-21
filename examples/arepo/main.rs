#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use std::path::PathBuf;

use bevy::prelude::*;
use mpi::traits::Equivalence;
use mpi::Rank;
use ordered_float::OrderedFloat;
use raxiom::components;
use raxiom::components::Density;
use raxiom::components::Position;
use raxiom::io::input::DatasetInputPlugin;
use raxiom::io::DatasetDescriptor;
use raxiom::io::DatasetShape;
use raxiom::io::InputDatasetDescriptor;
use raxiom::prelude::*;
use raxiom::units::Dimensionless;
use raxiom::units::Length;
use raxiom::units::PhotonFlux;
use raxiom::units::VecLength;

#[derive(Debug, Equivalence, Clone, PartialOrd, PartialEq)]
struct DistanceToSourceData {
    distance: Length,
    rank: Rank,
}

#[raxiom_parameters("postprocess")]
struct Parameters {
    initial_fraction_ionized_hydrogen: Dimensionless,
    source_strength: PhotonFlux,
}

fn read_vec(data: &[Float]) -> Position {
    Position(VecLength::new_unchecked(MVec::new(data[0], data[1])))
}

fn main() {
    let mut sim = SimulationBuilder::new();
    let mut sim = sim
        .parameters_from_relative_path(file!(), "parameters.yml")
        .headless(false)
        .write_output(true)
        .read_initial_conditions(true)
        .update_from_command_line_options()
        .build();
    sim.add_parameter_type::<Parameters>()
        .add_startup_system_to_stage(
            SimulationStartupStages::InsertDerivedComponents,
            insert_components_from_snapshot_system,
        )
        .add_startup_system_to_stage(
            SimulationStartupStages::InsertDerivedComponents,
            initialize_source_system,
        )
        .add_plugin(DatasetInputPlugin::<Position>::from_descriptor(
            InputDatasetDescriptor::<Position>::new(
                DatasetDescriptor {
                    dataset_name: "PartType0/Coordinates".into(),
                },
                DatasetShape::TwoDimensional(read_vec),
            ),
        ))
        .add_plugin(DatasetInputPlugin::<Density>::from_descriptor(
            InputDatasetDescriptor::<Density>::new(
                DatasetDescriptor {
                    dataset_name: "PartType0/Density".into(),
                },
                DatasetShape::OneDimensional,
            ),
        ))
        .add_plugin(CommunicationPlugin::<DistanceToSourceData>::default())
        .add_plugin(SweepPlugin)
        .run();
}

fn insert_components_from_snapshot_system(
    mut commands: Commands,
    particles: Particles<(Entity, &Position)>,
    parameters: Res<Parameters>,
) {
    for (entity, pos) in particles.iter() {
        commands
            .entity(entity)
            .insert((components::IonizedHydrogenFraction(
                parameters.initial_fraction_ionized_hydrogen,
            ),));
    }
}

fn initialize_source_system(
    mut commands: Commands,
    particles: Particles<(Entity, &Position)>,
    parameters: Res<Parameters>,
    box_size: Res<SimulationBox>,
    mut comm: Communicator<DistanceToSourceData>,
    world_rank: Res<WorldRank>,
) {
    let (closest_entity_to_center, distance) = particles
        .iter()
        .map(|(entity, pos)| {
            let dist = **pos - box_size.center();
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
            .entity(closest_entity_to_center)
            .insert(components::Source(parameters.source_strength));
    }
}
