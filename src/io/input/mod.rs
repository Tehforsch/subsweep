#[cfg(test)]
mod tests;

use std::marker::PhantomData;
use std::path::PathBuf;

use bevy::prelude::*;
use derive_custom::raxiom_parameters;
use hdf5::Dataset;
use hdf5::File;

use super::to_dataset::ToDataset;
use super::to_dataset::AMOUNT_IDENTIFIER;
use super::to_dataset::LENGTH_IDENTIFIER;
use super::to_dataset::MASS_IDENTIFIER;
use super::to_dataset::TEMPERATURE_IDENTIFIER;
use super::to_dataset::TIME_IDENTIFIER;
use crate::communication::WorldRank;
use crate::communication::WorldSize;
use crate::io::to_dataset::SCALE_FACTOR_IDENTIFIER;
use crate::named::Named;
use crate::prelude::LocalParticle;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;
use crate::units::Dimension;

/// Determines how a component is input into the simulation.
pub enum ComponentInput {
    /// The component needs to be present when the initial conditions are read.
    Required,
    /// The component does not need to be present and will be inserted
    /// by a startup system.
    Derived,
}

#[derive(Default, Deref, DerefMut, Resource)]
struct InputFiles(Vec<File>);

/// Parameters describing how the initial conditions
/// should be read. Only required if should_read_initial_conditions
/// is set in the [SimulationBuilder](crate::prelude::SimulationBuilder)
#[derive(Default)]
#[raxiom_parameters("input")]
pub struct InputParameters {
    /// The files containing the initial conditions
    pub paths: Vec<PathBuf>,
}

#[derive(Default, Deref, DerefMut, Resource)]
struct SpawnedEntities(Vec<Entity>);

#[derive(Named)]
pub struct DatasetInputPlugin<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for DatasetInputPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData::default(),
        }
    }
}

#[derive(SystemLabel)]
struct ReadDatasetLabel;

#[derive(Default, Deref, DerefMut, Resource)]
pub struct RegisteredDatasets(Vec<&'static str>);

impl<T: ToDataset + Component + Sync + Send + 'static> RaxiomPlugin for DatasetInputPlugin<T> {
    fn allow_adding_twice(&self) -> bool {
        true
    }

    fn should_build(&self, sim: &Simulation) -> bool {
        sim.read_initial_conditions
    }

    fn build_once_everywhere(&self, sim: &mut Simulation) {
        sim.add_parameter_type::<InputParameters>()
            .insert_resource(InputFiles::default())
            .insert_resource(SpawnedEntities::default())
            .add_startup_system(open_file_system)
            .add_startup_system(
                spawn_entities_system
                    .after(open_file_system)
                    .before(close_file_system),
            )
            .add_startup_system(close_file_system.after(spawn_entities_system));
    }

    fn build_everywhere(&self, sim: &mut Simulation) {
        let mut registered_datasets = sim.get_resource_or_insert_with(RegisteredDatasets::default);
        registered_datasets.push(T::name());
        sim.add_startup_system(
            read_dataset_system::<T>
                .after(open_file_system)
                .after(spawn_entities_system)
                .before(close_file_system)
                .label(ReadDatasetLabel)
                .ambiguous_with(ReadDatasetLabel),
        );
    }
}

fn open_file_system(
    mut files: ResMut<InputFiles>,
    parameters: Res<InputParameters>,
    rank: Res<WorldRank>,
    size: Res<WorldSize>,
) {
    let files_this_rank_should_open: Vec<_> = parameters
        .paths
        .iter()
        .enumerate()
        .filter(|(i, _)| i.rem_euclid(**size) == **rank as usize)
        .map(|(_, file)| file)
        .collect();
    assert!(files.is_empty());
    for path in files_this_rank_should_open.iter() {
        info!(
            "Reading initial conditions file: {}",
            path.to_str().unwrap()
        );
        files.push(File::open(path).unwrap_or_else(|_| {
            panic!(
                "Failed to open initial conditions file: {}",
                path.to_str().unwrap()
            )
        }));
    }
}

fn close_file_system(mut files: ResMut<InputFiles>) {
    files.0.clear();
}

fn spawn_entities_system(
    mut commands: Commands,
    mut spawned_entities: ResMut<SpawnedEntities>,
    datasets: Res<RegisteredDatasets>,
    files: Res<InputFiles>,
) {
    if datasets.len() == 0 {
        return;
    }
    let example_dataset = datasets[0];
    let get_num_entities = |dataset_name: &str| {
        files
            .iter()
            .map(|f| f.dataset(dataset_name).unwrap().shape()[0])
            .sum::<usize>()
    };
    let num_entities = get_num_entities(example_dataset);
    for dataset in datasets.iter() {
        let num_entities_this_dataset = get_num_entities(dataset);
        if num_entities_this_dataset != num_entities {
            panic!(
                "Different lengths of datasets: {example_dataset} ({num_entities}) and {dataset} ({num_entities_this_dataset})",
            );
        }
    }
    assert_eq!(spawned_entities.len(), 0);
    spawned_entities.0 = (0..num_entities)
        .map(|_| commands.spawn((LocalParticle,)).id())
        .collect();
}

fn read_dataset_system<T: ToDataset + Component>(
    mut commands: Commands,
    files: Res<InputFiles>,
    spawned_entities: Res<SpawnedEntities>,
) {
    let name = T::name();
    let data = files.iter().map(|file| {
        let set = file
            .dataset(name)
            .unwrap_or_else(|e| panic!("Failed to open dataset: {name}, {e:?}"));
        let data = set
            .read_1d::<T>()
            .unwrap_or_else(|e| panic!("Failed to read dataset: {name}, {e:?}"));
        let conversion_factor: f64 = set
            .attr(SCALE_FACTOR_IDENTIFIER)
            .expect("No scale factor in dataset")
            .read_scalar()
            .unwrap();
        assert_eq!(
            read_dimension(&set),
            T::dimension(),
            "Mismatch in dimension while reading dataset {name}.",
        );
        (data, conversion_factor)
    });
    for ((item, factor_written), entity) in data
        .flat_map(|(set, factor_written)| set.into_iter().map(move |item| (item, factor_written)))
        .zip(spawned_entities.iter())
    {
        let factor_read = T::dimension().base_conversion_factor();
        commands
            .entity(*entity)
            .insert(item.convert_base_units(factor_written / factor_read));
    }
}

fn read_dimension(dataset: &Dataset) -> Dimension {
    let read_attr = |ident, error_message| {
        dataset
            .attr(ident)
            .expect(error_message)
            .read_scalar()
            .unwrap()
    };
    let length: i32 = read_attr(LENGTH_IDENTIFIER, "No length scale factor in dataset");
    let mass: i32 = read_attr(MASS_IDENTIFIER, "No mass scale factor in dataset");
    let time: i32 = read_attr(TIME_IDENTIFIER, "No time scale factor in dataset");
    let temperature: i32 = read_attr(
        TEMPERATURE_IDENTIFIER,
        "No temperature scale factor in dataset",
    );
    let amount: i32 = read_attr(AMOUNT_IDENTIFIER, "No amount scale factor in dataset");
    Dimension {
        length,
        mass,
        time,
        temperature,
        amount,
    }
}
