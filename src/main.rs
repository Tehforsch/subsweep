#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod arepo_postprocess;

use arepo_postprocess::read_grid::ReadSweepGridPlugin;
use arepo_postprocess::remap::remap_abundances_and_energies_system;
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
use raxiom::components::IonizedHydrogenFraction;
use raxiom::components::Position;
use raxiom::cosmology::Cosmology;
use raxiom::io::input::DatasetInputPlugin;
use raxiom::io::DatasetDescriptor;
use raxiom::io::DatasetShape;
use raxiom::io::InputDatasetDescriptor;
use raxiom::prelude::*;
use raxiom::simulation_plugin::remove_components_system;
use raxiom::units::Dimensionless;
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
    match parameters.sources {
        SourceType::FromIcs(_) => {
            sim.add_startup_system(read_sources_system);
        }
        SourceType::SingleSource(_) => {
            if rank.is_main() {
                sim.add_startup_system(add_single_source_system);
            } else {
                sim.insert_resource(Sources::default());
            }
        }
    }
    match parameters.grid {
        GridParameters::Construct => sim.add_plugin(ParallelVoronoiGridConstruction),
        GridParameters::Read(_) => sim.add_plugin(ReadSweepGridPlugin),
    };
    if parameters.initial_fraction_ionized_hydrogen.is_none() {
        sim.add_plugin(DatasetInputPlugin::<ElectronAbundance>::from_descriptor(
            InputDatasetDescriptor::<ElectronAbundance> {
                descriptor: DatasetDescriptor {
                    dataset_name: "PartType0/ElectronAbundance".into(),
                    unit_reader: unit_reader.clone(),
                },
                ..default()
            },
        ));
    }
    sim.add_parameter_type::<Parameters>()
        .add_startup_system_to_stage(
            StartupStages::InsertComponentsAfterGrid,
            set_source_terms_system,
        )
        .add_startup_system_to_stage(
            StartupStages::ReadInput,
            insert_initial_ionized_fraction_system,
        )
        .add_startup_system_to_stage(
            StartupStages::InsertDerivedComponents,
            set_initial_ionized_fraction_from_electron_abundance_system,
        )
        .add_startup_system_to_stage(
            StartupStages::InsertDerivedComponents,
            insert_missing_components_system
                .after(set_initial_ionized_fraction_from_electron_abundance_system),
        )
        .add_startup_system_to_stage(StartupStages::Remap, remap_abundances_and_energies_system)
        .add_startup_system_to_stage(
            StartupStages::InsertGrid,
            remove_components_system::<InternalEnergy>,
        )
        .add_startup_system_to_stage(
            StartupStages::InsertGrid,
            remove_components_system::<ElectronAbundance>,
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
                    unit_reader: unit_reader,
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

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Default, Named)]
#[name = "electron_abundance"]
#[repr(transparent)]
pub struct ElectronAbundance(pub crate::units::Dimensionless);

fn insert_missing_components_system(
    mut commands: Commands,
    particles: Particles<(Entity, &IonizedHydrogenFraction, &InternalEnergy, &Density)>,
) {
    for (entity, ionized_hydrogen_fraction, internal_energy, density) in particles.iter() {
        let temperature = Temperature::from_internal_energy_density_hydrogen_only(
            **internal_energy * **density,
            **ionized_hydrogen_fraction,
            **density,
        );
        commands.entity(entity).insert((
            components::Rate(PhotonRate::zero()),
            components::Source(SourceRate::zero()),
            components::Temperature(temperature),
        ));
    }
}

fn insert_initial_ionized_fraction_system(
    mut commands: Commands,
    particles: Particles<Entity>,
    parameters: Res<Parameters>,
) {
    for entity in particles.iter() {
        let ionized_hydrogen_fraction = parameters
            .initial_fraction_ionized_hydrogen
            .unwrap_or(Dimensionless::dimensionless(0.0));
        commands
            .entity(entity)
            .insert((components::IonizedHydrogenFraction(
                ionized_hydrogen_fraction,
            ),));
    }
}

fn set_initial_ionized_fraction_from_electron_abundance_system(
    mut _particles: Particles<(&ElectronAbundance, &mut IonizedHydrogenFraction)>,
    parameters: Res<Parameters>,
) {
    if parameters.initial_fraction_ionized_hydrogen.is_none() {
        todo!("Fix the formula here - how exactly is xHII computed from ElectronAbundance? See TNG FAQ")
        // for (e, mut xhi) in particles.iter_mut() {
        // }
    }
}
