mod file_distribution;
#[cfg(test)]
mod tests;

use std::ops::Range;
use std::path::Path;
use std::path::PathBuf;

use bevy::prelude::debug;
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

use self::file_distribution::get_rank_assignment_for_rank;
use self::file_distribution::RankAssignment;
use self::file_distribution::Region;
use super::to_dataset::ToDataset;
use super::InputDatasetDescriptor;
use crate::communication::communicator::Communicator;
use crate::communication::Rank;
use crate::communication::SizedCommunicator;
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
            .insert_resource(SpawnedEntities::default())
            .add_startup_system(spawn_entities_system);
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
                    .after(spawn_entities_system)
                    .label(ReadDatasetLabel)
                    .ambiguous_with(ReadDatasetLabel),
            );
        }
    }
}

fn open_file(path: impl AsRef<Path>) -> File {
    File::open(path.as_ref())
        .unwrap_or_else(|_| panic!("Failed to open file: {}", path.as_ref().to_str().unwrap()))
}

pub struct Reader {
    rank: Rank,
    num_ranks: usize,
    files: Vec<File>,
}

impl Reader {
    /// Construct a reader with the contents of the files split evenly between the ranks.
    pub fn split_between_ranks<'a, U: AsRef<Path>>(paths: impl Iterator<Item = U>) -> Self {
        let t: Communicator<usize> = Communicator::new();
        let rank = t.rank();
        let num_ranks = t.size();
        Self {
            rank,
            num_ranks,
            files: paths.map(open_file).collect(),
        }
    }

    /// Construct a reader for the contents of all files.
    pub fn full<'a, U: AsRef<Path>>(paths: impl Iterator<Item = U>) -> Self {
        let rank = 0;
        let num_ranks = 1;
        Self {
            rank,
            num_ranks,
            files: paths.map(open_file).collect(),
        }
    }

    pub fn get_num_entities(&self, dataset_name: &str) -> usize {
        self.get_assignment(dataset_name)
            .regions
            .iter()
            .map(|region| region.size())
            .sum()
    }

    fn get_assignment(&self, dataset_name: &str) -> RankAssignment {
        let num_entries = self
            .files
            .iter()
            .map(|f| self.get_num_entries(dataset_name, &f))
            .collect::<Vec<_>>();
        get_rank_assignment_for_rank(&num_entries, self.num_ranks, self.rank)
    }

    fn get_num_entries(&self, dataset_name: &str, file: &File) -> usize {
        file.dataset(dataset_name).unwrap().shape()[0]
    }

    pub fn read_dataset<'a, T: ToDataset + Named>(
        &'a self,
        descriptor: InputDatasetDescriptor<T>,
    ) -> impl Iterator<Item = T> + 'a {
        let assignment = self.get_assignment(descriptor.dataset_name());
        assignment
            .regions
            .into_iter()
            .flat_map(move |region| self.read_region(descriptor.clone(), &region))
    }

    pub fn read_dataset_chunked<'a, 'b, T>(
        &'a self,
        descriptor: InputDatasetDescriptor<T>,
        chunk_size: usize,
    ) -> impl Iterator<Item = T> + 'a
    where
        T: ToDataset + Named,
    {
        let assignment = self.get_assignment(descriptor.dataset_name());
        assignment.regions.into_iter().flat_map(move |region| {
            self.read_region_chunked(descriptor.clone(), &region, chunk_size)
        })
    }

    fn read_region<'a, T: ToDataset>(
        &'a self,
        descriptor: InputDatasetDescriptor<T>,
        region: &Region,
    ) -> impl Iterator<Item = T> + 'a {
        self.read_region_chunked(descriptor, region, region.size())
    }

    fn read_region_chunked<'a, T: ToDataset>(
        &'a self,
        descriptor: InputDatasetDescriptor<T>,
        region: &Region,
        chunk_size: usize,
    ) -> impl Iterator<Item = T> + 'a {
        let factor_read = T::dimension().base_conversion_factor();
        let (set, factor_written) =
            get_dataset_and_conversion_factor_for_file(&descriptor, &self.files[region.file_index]);
        let chunks = ChunkIter::new(set, &descriptor, chunk_size, region);
        chunks.into_iter().flat_map(move |chunk| {
            convert_dataset_units(chunk, factor_read, factor_written).into_iter()
        })
    }
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

fn spawn_entities_system(
    mut commands: Commands,
    mut spawned_entities: ResMut<SpawnedEntities>,
    datasets: Res<RegisteredDatasets>,
    parameters: Res<InputParameters>,
) {
    let reader = Reader::split_between_ranks(parameters.paths.iter());
    if datasets.len() == 0 {
        return;
    }
    let (_, example_dataset) = &datasets.iter().next().unwrap();
    let num_entities = reader.get_num_entities(&example_dataset.name);
    for (_, dataset) in datasets.iter() {
        let num_entities_this_dataset = reader.get_num_entities(&dataset.name);
        if num_entities_this_dataset != num_entities {
            panic!(
                "Different lengths of datasets: {} ({num_entities}) and {} ({num_entities_this_dataset})", &example_dataset.name, &dataset.name
            );
        }
    }
    let mut comm: Communicator<usize> = Communicator::new();
    let num_entities_total: usize = comm.all_gather_sum(&num_entities);
    debug!("Spawned {} new entities", num_entities_total);
    assert_eq!(spawned_entities.len(), 0);
    spawned_entities.0 = (0..num_entities)
        .map(|_| commands.spawn((LocalParticle,)).id())
        .collect();
}

fn read_dataset_system<T: ToDataset + Component + Named>(
    descriptor: NonSend<InputDatasetDescriptor<T>>,
    mut commands: Commands,
    spawned_entities: Res<SpawnedEntities>,
    parameters: Res<InputParameters>,
) {
    let reader = Reader::split_between_ranks(parameters.paths.iter());
    for (item, entity) in reader
        .read_dataset::<T>(descriptor.clone())
        .enumerate()
        .map(|(_, t)| t)
        .zip(spawned_entities.iter())
    {
        commands.entity(*entity).insert(item);
    }
}

type Chunk<T> = ArrayBase<OwnedRepr<T>, Dim<[usize; 1]>>;

struct ChunkIter<T> {
    set: Dataset,
    slices: Vec<Range<usize>>,
    descriptor: InputDatasetDescriptor<T>,
}

fn get_chunk_sizes(region: &Region, chunk_size: usize) -> Vec<Range<usize>> {
    let dataset_size = region.size();
    let num_chunks = (dataset_size / chunk_size)
        + if dataset_size.rem_euclid(chunk_size) > 0 {
            1
        } else {
            0
        };
    (0..num_chunks)
        .map(|i| {
            let start = region.start + i * chunk_size;
            let end = (region.start + (i + 1) * chunk_size).min(region.end);
            start..end
        })
        .collect()
}

impl<T: ToDataset> ChunkIter<T> {
    fn new(
        set: Dataset,
        descriptor: &InputDatasetDescriptor<T>,
        chunk_size: usize,
        region: &Region,
    ) -> Self {
        let chunks = get_chunk_sizes(region, chunk_size);
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
    use crate::io::input::file_distribution::Region;

    #[test]
    fn get_chunk_sizes() {
        assert_eq!(
            super::get_chunk_sizes(
                &Region {
                    file_index: 0,
                    start: 0,
                    end: 450
                },
                100
            ),
            vec![0..100, 100..200, 200..300, 300..400, 400..450]
        );
        assert_eq!(
            super::get_chunk_sizes(
                &Region {
                    file_index: 0,
                    start: 0,
                    end: 400
                },
                100
            ),
            vec![0..100, 100..200, 200..300, 300..400]
        );
        assert_eq!(
            super::get_chunk_sizes(
                &Region {
                    file_index: 0,
                    start: 30,
                    end: 420
                },
                100
            ),
            vec![30..130, 130..230, 230..330, 330..420]
        );
        assert_eq!(
            super::get_chunk_sizes(
                &Region {
                    file_index: 0,
                    start: 20,
                    end: 420
                },
                100
            ),
            vec![20..120, 120..220, 220..320, 320..420]
        );
    }
}
