use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Entity;
use bevy::prelude::Res;
use bevy::prelude::Resource;
use derive_custom::Named;
use derive_more::Deref;
use derive_more::DerefMut;
use derive_more::From;
use hdf5::H5Type;
use mpi::traits::Equivalence;
use ordered_float::OrderedFloat;
use raxiom::communication::CommunicatedOption;
use raxiom::communication::Identified;
use raxiom::components;
use raxiom::components::Position;
use raxiom::io::input::read_dataset;
use raxiom::io::input::InputFiles;
use raxiom::io::DatasetDescriptor;
use raxiom::io::DatasetShape;
use raxiom::io::InputDatasetDescriptor;
use raxiom::mpidbg;
use raxiom::prelude::Communicator;
use raxiom::prelude::Particles;
use raxiom::prelude::SimulationBox;
use raxiom::units::Dimensionless;
use raxiom::units::Length;
use raxiom::units::Mass;
use raxiom::units::SourceRate;
use raxiom::units::Time;
use raxiom::units::VecLength;

use crate::cosmology::Cosmology;
use crate::read_vec;
use crate::unit_reader::ArepoUnitReader;
use crate::Parameters;

#[derive(Debug, Equivalence, Clone, PartialOrd, PartialEq)]
pub struct DistanceToSourceData(Length);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named)]
#[name = "metallicity"]
#[repr(transparent)]
pub struct Metallicity(pub Dimensionless);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named)]
#[name = "stellar_formation_time"]
#[repr(transparent)]
// This is dimensionless in the arepo outputs, since its the scale factor
pub struct StellarFormationTime(pub Dimensionless);

#[derive(Clone, Debug, Equivalence)]
pub struct Source {
    position: VecLength,
    age: Time,
    metallicity: Dimensionless,
    mass: Mass,
}

#[derive(Resource)]
pub struct Sources {
    sources: Vec<Source>,
}

fn make_descriptor<T>(
    unit_reader: &ArepoUnitReader,
    name: &str,
    shape: DatasetShape<T>,
) -> InputDatasetDescriptor<T> {
    InputDatasetDescriptor::<T> {
        descriptor: DatasetDescriptor {
            dataset_name: name.into(),
            unit_reader: Box::new(unit_reader.clone()),
        },
        shape,
    }
}

fn read_sources(files: &InputFiles, cosmology: &Cosmology) -> Vec<Source> {
    let unit_reader = ArepoUnitReader::new(cosmology.clone());
    let descriptor = &make_descriptor::<Position>(
        &unit_reader,
        "PartType4/Coordinates",
        DatasetShape::TwoDimensional(read_vec),
    );
    let position = read_dataset(&descriptor, files);
    let descriptor = &make_descriptor::<Metallicity>(
        &unit_reader,
        "PartType4/GFM_Metallicity",
        DatasetShape::OneDimensional,
    );
    let metallicity = read_dataset(&descriptor, files);
    let descriptor = &make_descriptor::<StellarFormationTime>(
        &unit_reader,
        "PartType4/GFM_StellarFormationTime",
        DatasetShape::OneDimensional,
    );
    let formation_time = read_dataset(&descriptor, files);
    let descriptor = &make_descriptor::<components::Mass>(
        &unit_reader,
        "PartType4/Masses",
        DatasetShape::OneDimensional,
    );
    let mass = read_dataset(&descriptor, files);
    position
        .zip(metallicity)
        .zip(formation_time)
        .zip(mass)
        .map(|(((position, metallicity), formation_time), mass)| {
            let age = Time::zero();
            Source {
                position: *position,
                metallicity: *metallicity,
                mass: *mass,
                age,
            }
        })
        .collect()
}

pub fn read_sources_system(
    mut commands: Commands,
    files: Res<InputFiles>,
    cosmology: Res<Cosmology>,
) {
    let sources = read_sources(&files, &cosmology);
    commands.insert_resource(Sources { sources });
}

pub fn initialize_sources_system(
    mut commands: Commands,
    particles: Particles<(Entity, &Position)>,
    parameters: Res<Parameters>,
    box_size: Res<SimulationBox>,
    mut source_comm: Communicator<Source>,
    mut comm: Communicator<CommunicatedOption<Identified<DistanceToSourceData>>>,
    sources: Res<Sources>,
) {
    let all_sources = source_comm.all_gather_varcount(&sources.sources);
    for (entity, _) in particles.iter() {
        commands
            .entity(entity)
            .insert(components::Source(SourceRate::zero()));
    }
    // let closest = find_closest_entity_for_each_source();
    // let closest = particles
    //     .iter()
    //     .map(|(entity, pos)| {
    //         let dist = **pos - box_size.center();
    //         (entity, OrderedFloat(dist.length().value_unchecked()))
    //     })
    //     .min_by_key(|(_, dist)| *dist)
    //     .map(|(entity, dist)| {
    //         Identified::new(entity, DistanceToSourceData(Length::new_unchecked(*dist)))
    //     });
    // let closest_on_each_rank = if let Some(closest) = closest {
    //     comm.all_gather(&Some(closest).into())
    // } else {
    //     comm.all_gather(&None.into())
    // };
    // let global_closest: Identified<DistanceToSourceData> = closest_on_each_rank
    //     .into_iter()
    //     .filter_map(|x| Into::<Option<_>>::into(x))
    //     .min_by(
    //         |x: &Identified<DistanceToSourceData>, y: &Identified<DistanceToSourceData>| {
    //             x.data
    //                 .partial_cmp(&y.data)
    //                 .unwrap_or(std::cmp::Ordering::Equal)
    //         },
    //     )
    //     .unwrap();
    // for (entity, _) in particles.iter() {
    //     let source = if entity == global_closest.entity() {
    //         parameters.source_strength
    //     } else {
    //         SourceRate::zero()
    //     };
    //     commands.entity(entity).insert(components::Source(source));
    // }
}
