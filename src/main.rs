#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![allow(non_local_definitions)]

mod arepo_postprocess;
mod emit_build_information;

use arepo_postprocess::read_grid::ReadSweepGridPlugin;
use arepo_postprocess::remap::remap_abundances_and_energies_system;
use arepo_postprocess::sources::read_sources_system;
use arepo_postprocess::unit_reader::read_vec;
use arepo_postprocess::unit_reader::ArepoUnitReader;
use arepo_postprocess::GridParameters;
use arepo_postprocess::Parameters;
use arepo_postprocess::SourceType;
use bevy_ecs::prelude::*;
use derive_more::Deref;
use derive_more::DerefMut;
use derive_more::From;
use emit_build_information::emit_build_information;
use hdf5::H5Type;
use mpi::traits::Equivalence;
use subsweep::components;
use subsweep::components::Density;
use subsweep::components::IonizedHydrogenFraction;
use subsweep::components::Position;
use subsweep::cosmology::Cosmology;
use subsweep::impl_to_dataset;
use subsweep::io::DatasetShape;
use subsweep::io::DefaultUnitReader;
use subsweep::parameters::OutputParameters;
use subsweep::prelude::*;
use subsweep::simulation_plugin::remove_components_system;
use subsweep::source_systems::SourcePlugin;
use subsweep::source_systems::Sources;
use subsweep::sweep::grid::Cell;
use subsweep::units::Dimensionless;
use subsweep::units::Mass;
use subsweep::units::PhotonRate;
use subsweep::units::SourceRate;

fn main() {
    let mut sim = SimulationBuilder::new();
    let mut sim = sim
        .write_output(true)
        .read_initial_conditions(true)
        .require_parameter_file(true)
        .update_from_command_line_options()
        .build();
    emit_build_information(&sim.get_resource::<OutputParameters>().unwrap());
    let parameters = sim
        .add_parameter_type_and_get_result::<Parameters>()
        .clone();
    let cosmology = sim.add_parameter_type_and_get_result::<Cosmology>().clone();
    let rank = sim.get_resource::<WorldRank>().unwrap();
    match &parameters.sources {
        SourceType::FromIcs(_) => {
            sim.add_startup_system(read_sources_system);
        }
        SourceType::Explicit(sources) => {
            if rank.is_main() {
                sim.insert_resource(Sources {
                    sources: sources.clone(),
                });
            } else {
                sim.insert_resource(Sources::default());
            }
        }
    }
    match parameters.grid {
        GridParameters::Construct => sim.add_plugin(ParallelVoronoiGridConstruction),
        GridParameters::Read(_) => sim.add_plugin(ReadSweepGridPlugin),
    };
    add_inputs(&mut sim, &parameters, cosmology);

    sim.add_plugin(SourcePlugin)
        .add_parameter_type::<Parameters>();
    if parameters.resume_from_subsweep {
        sim.add_startup_system_to_stage(
            StartupStages::ReadInput,
            insert_initial_ionized_fraction_system_subsweep,
        )
        .add_startup_system_to_stage(
            StartupStages::InsertDerivedComponents,
            insert_missing_components_system_subsweep,
        );
    } else {
        sim.add_startup_system_to_stage(
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
        );
    }
    sim.add_startup_system_to_stage(
        StartupStages::InsertComponentsAfterGrid,
        compute_cell_mass_system,
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
    .add_plugin(SweepPlugin)
    .run();
}

fn add_inputs(sim: &mut Simulation, parameters: &Parameters, cosmology: Cosmology) {
    if parameters.resume_from_subsweep {
        let unit_reader = DefaultUnitReader;
        sim.add_input_plugin::<IonizedHydrogenFraction>(
            "ionized_hydrogen_fraction",
            &unit_reader,
            None,
        );

        sim.add_input_plugin::<Position>("position", &unit_reader, None);
        sim.add_input_plugin::<Density>("density", &unit_reader, None);
        sim.add_input_plugin::<components::Temperature>("temperature", &unit_reader, None);
    } else {
        let unit_reader = ArepoUnitReader::new(cosmology);
        if parameters.initial_fraction_ionized_hydrogen.is_none() {
            sim.add_input_plugin::<ElectronAbundance>(
                "PartType0/ElectronAbundance",
                &unit_reader,
                None,
            );
        }

        sim.add_input_plugin::<Position>(
            "PartType0/Coordinates",
            &unit_reader,
            Some(DatasetShape::TwoDimensional(read_vec)),
        );
        sim.add_input_plugin::<Density>("PartType0/Density", &unit_reader, None);
        sim.add_input_plugin::<InternalEnergy>("PartType0/InternalEnergy", &unit_reader, None);
    }
}

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Default, Named)]
#[name = "internal_energy"]
#[repr(transparent)]
pub struct InternalEnergy(pub crate::units::EnergyPerMass);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Default, Named)]
#[name = "electron_abundance"]
#[repr(transparent)]
pub struct ElectronAbundance(pub crate::units::Dimensionless);

impl_to_dataset!(InternalEnergy, crate::units::EnergyPerMass, false);
impl_to_dataset!(ElectronAbundance, crate::units::Dimensionless, false);

fn insert_missing_components_system_subsweep(mut commands: Commands, particles: Particles<Entity>) {
    for entity in particles.iter() {
        commands.entity(entity).insert((
            components::PhotonRate(PhotonRate::zero()),
            components::Source(SourceRate::zero()),
            // Will be computed later
            components::Mass(Mass::zero()),
        ));
    }
}

fn insert_missing_components_system(
    mut commands: Commands,
    particles: Particles<(Entity, &IonizedHydrogenFraction, &InternalEnergy, &Density)>,
) {
    for (entity, ionized_hydrogen_fraction, internal_energy, density) in particles.iter() {
        let temperature = units::Temperature::from_internal_energy_density_hydrogen_only(
            **internal_energy * **density,
            **ionized_hydrogen_fraction,
            **density,
        );
        commands.entity(entity).insert((
            components::PhotonRate(PhotonRate::zero()),
            components::Source(SourceRate::zero()),
            components::Temperature(temperature),
            // Will be computed later
            components::Mass(Mass::zero()),
        ));
    }
}

fn insert_initial_ionized_fraction_system_subsweep(
    mut commands: Commands,
    particles: Particles<Entity>,
) {
    for entity in particles.iter() {
        commands
            .entity(entity)
            .insert(components::DeltaIonizedHydrogenFraction(0.0.into()));
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
            .insert((components::DeltaIonizedHydrogenFraction(0.0.into()),))
            .insert((components::IonizedHydrogenFraction(
                ionized_hydrogen_fraction,
            ),));
    }
}

fn compute_cell_mass_system(
    mut commands: Commands,
    particles: Particles<(Entity, &Density, &Cell)>,
) {
    for (entity, dens, cell) in particles.iter() {
        let mass = **dens * cell.volume();
        commands.entity(entity).insert(components::Mass(mass));
    }
}

fn set_initial_ionized_fraction_from_electron_abundance_system(
    mut particles: Particles<(&ElectronAbundance, &mut IonizedHydrogenFraction)>,
    parameters: Res<Parameters>,
) {
    if parameters.initial_fraction_ionized_hydrogen.is_none() {
        // Assume this everywhere, to simplify matters. The initial ionization fractions here don't need
        // to be super accurate, since we remap them anyways.
        let xh = Dimensionless::dimensionless(0.76);
        for (xe, mut xhi) in particles.iter_mut() {
            **xhi = (xh * **xe).clamp(1e-10, 1.0 - 1e-10);
        }
    }
}
