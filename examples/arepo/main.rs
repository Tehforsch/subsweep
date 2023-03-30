#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod unit_reader;

use bevy::prelude::*;
use mpi::traits::Equivalence;
use ordered_float::OrderedFloat;
use raxiom::communication::CommunicatedOption;
use raxiom::communication::Identified;
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
use raxiom::units::SourceRate;
use raxiom::units::VecLength;
use unit_reader::ArepoUnitReader;

#[derive(Debug, Equivalence, Clone, PartialOrd, PartialEq)]
struct DistanceToSourceData(Length);

#[raxiom_parameters("postprocess")]
struct Parameters {
    initial_fraction_ionized_hydrogen: Dimensionless,
    source_strength: PhotonFlux,
}

fn read_vec(data: &[Float]) -> Position {
    Position(VecLength::new_unchecked(MVec::new(
        data[0], data[1], data[2],
    )))
}

fn main() {
    let mut sim = SimulationBuilder::new();
    let mut sim = sim
        .parameters_from_relative_path(file!(), "parameters.yml")
        .headless(true)
        .write_output(true)
        .read_initial_conditions(true)
        .update_from_command_line_options()
        .build();
    sim.add_parameter_type::<Parameters>()
        .add_startup_system_to_stage(
            SimulationStartupStages::InsertDerivedComponents,
            insert_missing_components_system,
        )
        .add_startup_system_to_stage(
            SimulationStartupStages::InsertDerivedComponents,
            initialize_source_system,
        )
        .add_plugin(ParallelVoronoiGridConstruction)
        .add_plugin(DatasetInputPlugin::<Position>::from_descriptor(
            InputDatasetDescriptor::<Position>::new(
                DatasetDescriptor {
                    dataset_name: "PartType0/Coordinates".into(),
                    unit_reader: Box::new(ArepoUnitReader),
                },
                DatasetShape::TwoDimensional(read_vec),
            ),
        ))
        .add_plugin(DatasetInputPlugin::<Density>::from_descriptor(
            InputDatasetDescriptor::<Density> {
                descriptor: DatasetDescriptor {
                    dataset_name: "PartType0/Density".into(),
                    unit_reader: Box::new(ArepoUnitReader),
                },
                ..default()
            },
        ))
        .add_plugin(CommunicationPlugin::<
            CommunicatedOption<Identified<DistanceToSourceData>>,
        >::default())
        .add_plugin(SweepPlugin)
        .run();
}

fn insert_missing_components_system(
    mut commands: Commands,
    particles: Particles<(Entity, &Position)>,
    parameters: Res<Parameters>,
) {
    for (entity, _) in particles.iter() {
        commands.entity(entity).insert((
            components::IonizedHydrogenFraction(parameters.initial_fraction_ionized_hydrogen),
            components::Flux(PhotonFlux::zero()),
        ));
    }
}

fn initialize_source_system(
    mut commands: Commands,
    particles: Particles<(Entity, &Position)>,
    parameters: Res<Parameters>,
    box_size: Res<SimulationBox>,
    mut comm: Communicator<CommunicatedOption<Identified<DistanceToSourceData>>>,
) {
    let closest = particles
        .iter()
        .map(|(entity, pos)| {
            let dist = **pos - box_size.center();
            (entity, OrderedFloat(dist.length().value_unchecked()))
        })
        .min_by_key(|(_, dist)| *dist)
        .map(|(entity, dist)| {
            Identified::new(entity, DistanceToSourceData(Length::new_unchecked(*dist)))
        });
    let closest_on_each_rank = if let Some(closest) = closest {
        comm.all_gather(&Some(closest).into())
    } else {
        comm.all_gather(&None.into())
    };
    let global_closest: Identified<DistanceToSourceData> = closest_on_each_rank
        .into_iter()
        .filter_map(|x| Into::<Option<_>>::into(x))
        .min_by(
            |x: &Identified<DistanceToSourceData>, y: &Identified<DistanceToSourceData>| {
                x.data
                    .partial_cmp(&y.data)
                    .unwrap_or(std::cmp::Ordering::Equal)
            },
        )
        .unwrap();
    for (entity, _) in particles.iter() {
        let source = if entity == global_closest.entity() {
            parameters.source_strength
        } else {
            SourceRate::zero()
        };
        commands.entity(entity).insert(components::Source(source));
    }
}
