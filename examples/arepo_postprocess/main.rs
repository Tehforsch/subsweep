#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod bpass;
mod sources;
mod unit_reader;

use bevy::prelude::*;
use derive_more::From;
use hdf5::H5Type;
use mpi::traits::Equivalence;
use raxiom::components;
use raxiom::components::Density;
use raxiom::components::Position;
use raxiom::cosmology::Cosmology;
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
use raxiom::units::Temperature;
use raxiom::units::VecLength;
use sources::add_single_source_system;
use sources::read_sources_system;
use sources::set_source_terms_system;
use unit_reader::ArepoUnitReader;

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Default, Named)]
#[name = "internal_energy"]
#[repr(transparent)]
pub struct InternalEnergy(pub crate::units::EnergyPerMass);

#[raxiom_parameters("postprocess")]
pub struct Parameters {
    initial_fraction_ionized_hydrogen: Dimensionless,
    sources: SourceType,
}

#[derive(Default)]
#[raxiom_parameters]
enum SourceType {
    #[default]
    FromIcs,
    SingleSource(SourceRate),
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
    match parameters.sources {
        SourceType::FromIcs => {
            sim.add_startup_system(
                read_sources_system
                    .after(open_file_system)
                    .before(close_file_system),
            );
        }
        SourceType::SingleSource(_) => {
            sim.add_startup_system(add_single_source_system);
        }
    }
    sim.add_parameter_type::<Parameters>()
        .add_startup_system_to_stage(
            SimulationStartupStages::InsertComponentsAfterGrid,
            set_source_terms_system,
        )
        .add_startup_system_to_stage(
            SimulationStartupStages::InsertDerivedComponents,
            insert_missing_components_system,
        )
        .add_startup_system_to_stage(
            SimulationStartupStages::InsertGrid,
            remove_unnecessary_components_system,
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
        .add_plugin(DatasetInputPlugin::<InternalEnergy>::from_descriptor(
            InputDatasetDescriptor::<InternalEnergy> {
                descriptor: DatasetDescriptor {
                    dataset_name: "PartType0/InternalEnergy".into(),
                    unit_reader: unit_reader.clone(),
                },
                ..default()
            },
        ))
        .add_plugin(SweepPlugin)
        .run();
}

fn insert_missing_components_system(
    mut commands: Commands,
    particles: Particles<(Entity, &InternalEnergy, &Density)>,
    parameters: Res<Parameters>,
) {
    for (entity, internal_energy, density) in particles.iter() {
        let ionized_hydrogen_fraction = parameters.initial_fraction_ionized_hydrogen;
        let temperature = Temperature::from_internal_energy_density_hydrogen_only(
            **internal_energy * **density,
            ionized_hydrogen_fraction,
            **density,
        );
        commands.entity(entity).insert((
            components::IonizedHydrogenFraction(ionized_hydrogen_fraction),
            components::Flux(PhotonFlux::zero()),
            components::Source(SourceRate::zero()),
            components::Temperature(temperature),
        ));
    }
}

fn remove_unnecessary_components_system(
    mut commands: Commands,
    particles: Particles<Entity, With<InternalEnergy>>,
) {
    for entity in particles.iter() {
        commands.entity(entity).remove::<InternalEnergy>();
    }
}
