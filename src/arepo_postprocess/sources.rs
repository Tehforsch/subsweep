use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Res;
use derive_custom::Named;
use derive_more::Deref;
use derive_more::DerefMut;
use derive_more::From;
use hdf5::H5Type;
use mpi::traits::Equivalence;
use raxiom::components;
use raxiom::components::Position;
use raxiom::cosmology::Cosmology;
use raxiom::io::input::Reader;
use raxiom::io::DatasetShape;
use raxiom::parameters::InputParameters;
use raxiom::prelude::SimulationBox;
use raxiom::source_systems::Source;
use raxiom::source_systems::Sources;
use raxiom::units::Dimensionless;
use raxiom::units::Mass;
use raxiom::units::Time;
use raxiom::units::VecLength;

use super::bpass::bpass_lookup;
use super::unit_reader::make_descriptor;
use super::unit_reader::read_vec;
use super::unit_reader::ArepoUnitReader;
use super::Parameters;
use super::SourceType;

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named)]
#[name = "metallicity"]
#[repr(transparent)]
pub struct Metallicity(pub Dimensionless);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named)]
#[name = "stellar_formation_time"]
#[repr(transparent)]
// This is dimensionless in the arepo outputs, since its the scale factor
pub struct StellarFormationTime(pub Dimensionless);

pub fn add_single_source_system(
    box_size: Res<SimulationBox>,
    parameters: Res<Parameters>,
    mut commands: Commands,
) {
    let center = box_size.center;
    if let SourceType::SingleSource(rate) = parameters.sources {
        commands.insert_resource(Sources {
            sources: vec![Source {
                position: center,
                rate,
            }],
        })
    }
}

pub fn read_sources_system(
    mut commands: Commands,
    parameters: Res<InputParameters>,
    run_parameters: Res<Parameters>,
    cosmology: Res<Cosmology>,
) {
    let reader = Reader::split_between_ranks(parameters.all_input_files());
    let from_ics = run_parameters.sources.unwrap_from_ics();
    let sources = read_sources(&reader, &cosmology, from_ics.escape_fraction);
    commands.insert_resource(Sources { sources });
}

fn new_bpass_source(
    cosmology: &Cosmology,
    position: VecLength,
    metallicity: Dimensionless,
    mass: Mass,
    formation_scale_factor: Dimensionless,
    escape_fraction: Dimensionless,
) -> Source {
    let age = formation_scale_factor_to_age(cosmology, formation_scale_factor);
    Source {
        position,
        rate: bpass_lookup(age, metallicity, mass) * escape_fraction,
    }
}

fn formation_scale_factor_to_age(
    cosmology: &Cosmology,
    formation_scale_factor: Dimensionless,
) -> Time {
    cosmology.time_difference_between_scalefactors(formation_scale_factor, cosmology.scale_factor())
}

fn read_sources(
    reader: &Reader,
    cosmology: &Cosmology,
    escape_fraction: Dimensionless,
) -> Vec<Source> {
    let unit_reader = ArepoUnitReader::new(cosmology.clone());
    let descriptor = make_descriptor::<Position, _>(
        &unit_reader,
        "PartType4/Coordinates",
        DatasetShape::TwoDimensional(read_vec),
    );
    let position = reader.read_dataset(descriptor);
    let descriptor = make_descriptor::<Metallicity, _>(
        &unit_reader,
        "PartType4/GFM_Metallicity",
        DatasetShape::OneDimensional,
    );
    let metallicity = reader.read_dataset(descriptor);
    let descriptor = make_descriptor::<StellarFormationTime, _>(
        &unit_reader,
        "PartType4/GFM_StellarFormationTime",
        DatasetShape::OneDimensional,
    );
    let formation_scale_factor = reader.read_dataset(descriptor);
    let descriptor = make_descriptor::<components::Mass, _>(
        &unit_reader,
        "PartType4/Masses",
        DatasetShape::OneDimensional,
    );
    let mass = reader.read_dataset(descriptor);
    position
        .zip(metallicity)
        .zip(formation_scale_factor)
        .zip(mass)
        // Everything else is WIND. Love the data structures in Arepo
        .filter(|(((_, _), formation_scale_factor), _)| formation_scale_factor.is_positive())
        .map(
            |(((position, metallicity), formation_scale_factor), mass)| {
                new_bpass_source(
                    cosmology,
                    *position,
                    *metallicity,
                    *mass,
                    *formation_scale_factor,
                    escape_fraction,
                )
            },
        )
        .collect()
}
