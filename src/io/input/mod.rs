#[cfg(test)]
mod tests;

use std::ops::Range;
use std::path::PathBuf;

use bevy::prelude::debug;
use bevy::prelude::info;
use bevy::prelude::warn;
use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use bevy::prelude::Entity;
use bevy::prelude::IntoSystemDescriptor;
use bevy::prelude::NonSend;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::Resource;
use bevy::prelude::SystemLabel;
use derive_custom::raxiom_parameters;
use hdf5::Dataset;
use hdf5::File;
use hdf5::Result;
use hdf5::Selection;
use ndarray::s;
use ndarray::ArrayBase;
use ndarray::Dim;
use ndarray::OwnedRepr;

use super::to_dataset::ToDataset;
use super::InputDatasetDescriptor;
use crate::communication::WorldRank;
use crate::communication::WorldSize;
use crate::hash_map::HashMap;
use crate::io::DatasetShape;
use crate::prelude::Float;
use crate::prelude::LocalParticle;
use crate::prelude::Named;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;

/// Determines how a component is input into the simulation.
pub enum ComponentInput<T> {
    /// The component needs to be present in the given dataset when the initial conditions are read.
    Required(InputDatasetDescriptor<T>),
    /// The component does not need to be present and will be inserted
    /// by a startup system.
    Derived,
}

#[derive(Default, Deref, DerefMut, Resource)]
pub struct InputFiles(Vec<File>);

/// Parameters describing how the initial conditions
/// should be read. Only required if should_read_initial_conditions
/// is set in the [SimulationBuilder](crate::prelude::SimulationBuilder)
#[derive(Default)]
#[raxiom_parameters("input")]
pub struct InputParameters {
    /// The files containing the initial conditions
    pub paths: Vec<PathBuf>,
    /// Utility for debugging: "Shrink" the ICS by only using every
    /// nth particle.
    pub shrink_factor: Option<usize>,
}

#[derive(Default, Deref, DerefMut, Resource)]
struct SpawnedEntities(Vec<Entity>);

#[derive(Named)]
pub struct DatasetInputPlugin<T> {
    descriptor: InputDatasetDescriptor<T>,
}

impl<T> DatasetInputPlugin<T> {
    pub fn from_descriptor(descriptor: InputDatasetDescriptor<T>) -> Self {
        Self { descriptor }
    }
}

#[derive(SystemLabel)]
struct ReadDatasetLabel;

#[derive(Default, Deref, DerefMut, Resource)]
pub struct RegisteredDatasets(HashMap<String, RegisteredDataset>);

#[derive(Default, Resource)]
pub struct RegisteredDataset {
    name: String,
}

impl<T: Named + ToDataset + Component + Sync + Send + 'static> RaxiomPlugin
    for DatasetInputPlugin<T>
{
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
        registered_datasets.insert(
            T::name().into(),
            RegisteredDataset {
                name: self.descriptor.dataset_name().into(),
            },
        );
        let input_plugin_for_type_been_added_previously = sim
            .get_non_send_resource::<InputDatasetDescriptor<T>>()
            .is_some();
        // Always use the last descriptor that has been added for a particular type.
        sim.insert_non_send_resource(self.descriptor.clone());
        // Only add read_dataset_system if it has not been added by another DatasetInputPlugin earlier.
        if !input_plugin_for_type_been_added_previously {
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
}

pub fn open_file_system(
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

pub fn close_file_system(mut files: ResMut<InputFiles>) {
    files.0.clear();
}

fn warn_if_shrink_factor_is_enabled(parameters: &InputParameters) {
    if let Some(shrink_factor) = parameters.shrink_factor {
        if shrink_factor != 1 {
            warn!(
                "Shrinking ICS by using only every {}-th particle",
                shrink_factor
            );
        }
    }
}

fn spawn_entities_system(
    mut commands: Commands,
    mut spawned_entities: ResMut<SpawnedEntities>,
    datasets: Res<RegisteredDatasets>,
    files: Res<InputFiles>,
    parameters: Res<InputParameters>,
) {
    if datasets.len() == 0 {
        return;
    }
    warn_if_shrink_factor_is_enabled(&parameters);
    let (_, example_dataset) = &datasets.iter().next().unwrap();
    let get_num_entities = |dataset_name: &str| {
        files
            .iter()
            .map(|f| f.dataset(dataset_name).unwrap().shape()[0])
            .sum::<usize>()
            / parameters.shrink_factor.unwrap_or(1)
    };
    let num_entities = get_num_entities(&example_dataset.name);
    for (_, dataset) in datasets.iter() {
        let num_entities_this_dataset = get_num_entities(&dataset.name);
        if num_entities_this_dataset != num_entities {
            panic!(
                "Different lengths of datasets: {} ({num_entities}) and {} ({num_entities_this_dataset})", &example_dataset.name, &dataset.name
            );
        }
    }
    debug!("Spawned {} new entities", num_entities);
    assert_eq!(spawned_entities.len(), 0);
    spawned_entities.0 = (0..num_entities)
        .map(|_| commands.spawn((LocalParticle,)).id())
        .collect();
}

fn read_dataset_system<T: ToDataset + Component>(
    descriptor: NonSend<InputDatasetDescriptor<T>>,
    mut commands: Commands,
    files: Res<InputFiles>,
    spawned_entities: Res<SpawnedEntities>,
    parameters: Res<InputParameters>,
) {
    let should_insert = |i: usize| {
        if let Some(shrink_factor) = parameters.shrink_factor {
            i.rem_euclid(shrink_factor) == 0
        } else {
            true
        }
    };
    for (item, entity) in read_dataset::<T>(&descriptor, &files)
        .enumerate()
        .filter(|(i, _)| should_insert(*i))
        .map(|(_, t)| t)
        .zip(spawned_entities.iter())
    {
        commands.entity(*entity).insert(item);
    }
}

type Chunk<T> = ArrayBase<OwnedRepr<T>, Dim<[usize; 1]>>;

pub fn read_dataset<'a, T: ToDataset>(
    descriptor: &'a InputDatasetDescriptor<T>,
    files: &'a InputFiles,
) -> impl Iterator<Item = T> + 'a {
    info!("Reading dataset {}", descriptor.dataset_name());
    files
        .iter()
        .flat_map(move |file| read_dataset_for_file(descriptor, file).into_iter())
}

pub fn read_dataset_for_file<'a, T: ToDataset>(
    descriptor: &'a InputDatasetDescriptor<T>,
    file: &'a File,
) -> Vec<T> {
    let factor_read = T::dimension().base_conversion_factor();
    let (set, factor_written) = get_dataset_and_conversion_factor_for_file(descriptor, file);
    let data = read_chunk(&set, descriptor, 0..set.shape()[0]);
    convert_dataset_units(data, factor_read, factor_written)
}

/// Iterate over the items in the dataset without reading the
/// entire dataset at once - instead, the dataset is read in chunks
/// of size chunk_size.
pub fn read_dataset_for_file_chunked<'a, T: ToDataset>(
    descriptor: &'a InputDatasetDescriptor<T>,
    file: &'a File,
    chunk_size: usize,
) -> impl Iterator<Item = T> {
    let factor_read = T::dimension().base_conversion_factor();
    let (set, factor_written) = get_dataset_and_conversion_factor_for_file(descriptor, &file);
    let chunks = ChunkIter::new(set, descriptor, chunk_size);
    chunks.into_iter().flat_map(move |chunk| {
        convert_dataset_units(chunk, factor_read, factor_written).into_iter()
    })
}

fn get_dataset_and_conversion_factor_for_file<'a, T: ToDataset>(
    descriptor: &'a InputDatasetDescriptor<T>,
    file: &'a File,
) -> (Dataset, f64) {
    let name = descriptor.dataset_name();
    let set = file
        .dataset(name)
        .unwrap_or_else(|e| panic!("Failed to open dataset: {name}, {e:?}"));
    let conversion_factor = descriptor.read_scale_factor(&set);
    assert_eq!(
        descriptor.read_dimension(&set),
        T::dimension(),
        "Mismatch in dimension while reading dataset {name}.",
    );
    (set, conversion_factor)
}

fn convert_dataset_units<T: ToDataset>(
    data: Chunk<T>,
    factor_read: f64,
    factor_written: f64,
) -> Vec<T> {
    data.into_iter()
        .map(|item| item.convert_base_units(factor_written / factor_read))
        .collect()
}

struct ChunkIter<T> {
    set: Dataset,
    slices: Vec<Range<usize>>,
    descriptor: InputDatasetDescriptor<T>,
}

fn get_chunk_sizes(dataset_size: usize, chunk_size: usize) -> Vec<Range<usize>> {
    let num_chunks = (dataset_size / chunk_size)
        + if dataset_size.rem_euclid(chunk_size) > 0 {
            1
        } else {
            0
        };
    (0..num_chunks)
        .map(|i| (i * chunk_size..((i + 1) * chunk_size).min(dataset_size)))
        .collect()
}

impl<T: ToDataset> ChunkIter<T> {
    fn new(set: Dataset, descriptor: &InputDatasetDescriptor<T>, chunk_size: usize) -> Self {
        let shape = set.shape();
        let chunks = get_chunk_sizes(shape[0], chunk_size);
        Self {
            set,
            slices: chunks,
            descriptor: descriptor.clone(),
        }
    }
}

impl<T: ToDataset> Iterator for ChunkIter<T> {
    type Item = Chunk<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.slices.is_empty() {
            None
        } else {
            let slice = self.slices.remove(0);
            Some(read_chunk(&self.set, &self.descriptor, slice))
        }
    }
}

fn read_chunk<T: ToDataset>(
    set: &Dataset,
    descriptor: &InputDatasetDescriptor<T>,
    slice: Range<usize>,
) -> Chunk<T> {
    read_chunk_fallible(set, descriptor, slice).unwrap_or_else(|e| {
        let name = descriptor.dataset_name();
        panic!("Failed to read dataset: {name}, {e:?}")
    })
}

fn read_chunk_fallible<T: ToDataset>(
    set: &Dataset,
    descriptor: &InputDatasetDescriptor<T>,
    slice: Range<usize>,
) -> Result<Chunk<T>> {
    Ok(match descriptor.shape {
        DatasetShape::OneDimensional => set.read_slice_1d::<T, _>(slice)?,
        DatasetShape::TwoDimensional(constructor) => set
            .read_slice_2d::<Float, _>(Selection::try_new(s![slice, ..]).unwrap())?
            .outer_iter()
            .map(|row| constructor(row.as_slice().unwrap()))
            .collect(),
    })
}

#[cfg(test)]
mod unit_tests {
    #[test]
    fn get_chunk_sizes() {
        assert_eq!(
            super::get_chunk_sizes(450, 100),
            vec![0..100, 100..200, 200..300, 300..400, 400..450]
        );
        assert_eq!(
            super::get_chunk_sizes(400, 100),
            vec![0..100, 100..200, 200..300, 300..400]
        );
    }
}
