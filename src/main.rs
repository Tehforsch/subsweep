#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod arepo_postprocess;

use arepo_postprocess::read_grid::ReadSweepGridPlugin;
use arepo_postprocess::sources::add_single_source_system;
use arepo_postprocess::sources::read_sources_system;
use arepo_postprocess::sources::set_source_terms_system;
use arepo_postprocess::sources::Sources;
use arepo_postprocess::unit_reader::read_vec;
use arepo_postprocess::unit_reader::ArepoUnitReader;
use arepo_postprocess::GridParameters;
use arepo_postprocess::Parameters;
use arepo_postprocess::SourceType;
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
use raxiom::simulation_plugin::remove_components_system;
use raxiom::units::PhotonRate;
use raxiom::units::SourceRate;
use raxiom::units::Temperature;

fn main() {
    let mut sim = SimulationBuilder::new();
    let mut sim = sim
        .write_output(true)
        .read_initial_conditions(true)
        .require_parameter_file(true)
        .update_from_command_line_options()
        .build();
    let cosmology = sim.add_parameter_type_and_get_result::<Cosmology>().clone();
    let unit_reader = Box::new(ArepoUnitReader::new(cosmology));
    let parameters = sim
        .add_parameter_type_and_get_result::<Parameters>()
        .clone();
    let rank = sim.get_resource::<WorldRank>().unwrap();
    if rank.is_main() {
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
    } else {
        sim.insert_resource(Sources::default());
    }
    match parameters.grid {
        GridParameters::Construct => sim.add_plugin(ParallelVoronoiGridConstruction),
        GridParameters::Read(_) => sim.add_plugin(ReadSweepGridPlugin),
    };
    sim.add_parameter_type::<Parameters>()
        .add_startup_system_to_stage(
            StartupStages::InsertComponentsAfterGrid,
            set_source_terms_system,
        )
        .add_startup_system_to_stage(
            StartupStages::InsertDerivedComponents,
            insert_missing_components_system,
        )
        .add_startup_system_to_stage(
            StartupStages::InsertGrid,
            remove_components_system::<InternalEnergy>,
        )
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
                    unit_reader,
                },
                ..default()
            },
        ))
        .add_plugin(SweepPlugin)
        .run();
}

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Default, Named)]
#[name = "internal_energy"]
#[repr(transparent)]
pub struct InternalEnergy(pub crate::units::EnergyPerMass);

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
            components::Rate(PhotonRate::zero()),
            components::Source(SourceRate::zero()),
            components::Temperature(temperature),
        ));
    }
}
