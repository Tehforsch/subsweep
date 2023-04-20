#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod bpass;
mod cosmology;
mod sources;
mod unit_reader;

use bevy::prelude::*;
use cosmology::Cosmology;
use raxiom::communication::CommunicatedOption;
use raxiom::communication::Identified;
use raxiom::components;
use raxiom::components::Density;
use raxiom::components::Position;
use raxiom::io::input::close_file_system;
use raxiom::io::input::open_file_system;
use raxiom::io::input::DatasetInputPlugin;
use raxiom::io::DatasetDescriptor;
use raxiom::io::DatasetShape;
use raxiom::io::InputDatasetDescriptor;
use raxiom::prelude::*;
use raxiom::units::Dimensionless;
use raxiom::units::PhotonFlux;
use raxiom::units::SourceRate;
use raxiom::units::VecLength;
use sources::read_sources_system;
use sources::set_source_terms_system;
use sources::DistanceToSourceData;
use sources::Source;
use unit_reader::ArepoUnitReader;

#[raxiom_parameters("postprocess")]
pub struct Parameters {
    initial_fraction_ionized_hydrogen: Dimensionless,
    sources_from_ics: bool,
}

fn read_vec(data: &[Float]) -> Position {
    Position(VecLength::new_unchecked(MVec::new(
        data[0], data[1], data[2],
    )))
}

fn main() {
    let mut sim = SimulationBuilder::new();
    let mut sim = sim
        .write_output(true)
        .read_initial_conditions(true)
        .update_from_command_line_options()
        .build();
    let cosmology = sim.add_parameter_type_and_get_result::<Cosmology>().clone();
    let unit_reader = Box::new(ArepoUnitReader::new(cosmology));
    let parameters = sim.add_parameter_type_and_get_result::<Parameters>();
    if parameters.sources_from_ics {
        sim.add_startup_system_to_stage(
            SimulationStartupStages::InsertComponentsAfterGrid,
            set_source_terms_system,
        )
        .add_startup_system(
            read_sources_system
                .after(open_file_system)
                .before(close_file_system),
        );
    }
    sim.add_parameter_type::<Parameters>()
        .add_startup_system_to_stage(
            SimulationStartupStages::InsertDerivedComponents,
            insert_missing_components_system,
        )
        .add_plugin(ParallelVoronoiGridConstruction)
        .add_plugin(DatasetInputPlugin::<Position>::from_descriptor(
            InputDatasetDescriptor::<Position>::new(
                DatasetDescriptor {
                    dataset_name: "PartType0/Coordinates".into(),
                    unit_reader: unit_reader.clone(),
                },
                DatasetShape::TwoDimensional(read_vec),
            ),
        ))
        .add_plugin(DatasetInputPlugin::<Density>::from_descriptor(
            InputDatasetDescriptor::<Density> {
                descriptor: DatasetDescriptor {
                    dataset_name: "PartType0/Density".into(),
                    unit_reader: unit_reader.clone(),
                },
                ..default()
            },
        ))
        .add_plugin(CommunicationPlugin::<
            CommunicatedOption<Identified<DistanceToSourceData>>,
        >::default())
        .add_plugin(CommunicationPlugin::<Source>::default())
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
            components::Source(SourceRate::zero()),
        ));
    }
}
