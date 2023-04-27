use std::fs;
use std::marker::PhantomData;
use std::path::Path;
use std::path::PathBuf;

use bevy::prelude::EventReader;
use bevy::prelude::IntoSystemDescriptor;
use bevy::prelude::NonSend;
use bevy::prelude::Res;
use hdf5::Dataset;
use hdf5::File;

use super::output::make_output_dirs_system;
use super::output::OutputStages;
use super::to_dataset::create_empty_dataset;
use super::to_dataset::ToDataset;
use super::DatasetDescriptor;
use super::OutputDatasetDescriptor;
use crate::named::Named;
use crate::parameters::OutputParameters;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;
use crate::simulation_plugin::SimulationTime;

pub const TIME_DATASET_IDENTIFIER: &str = "time";

pub trait TimeSeries: ToDataset + std::fmt::Debug {}

impl<T> TimeSeries for T where T: ToDataset + std::fmt::Debug {}

#[derive(Named)]
pub struct TimeSeriesPlugin<T: TimeSeries> {
    _marker: PhantomData<T>,
}

impl<T: TimeSeries> Default for TimeSeriesPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData::default(),
        }
    }
}

impl<T: TimeSeries> RaxiomPlugin for TimeSeriesPlugin<T> {
    fn allow_adding_twice(&self) -> bool {
        true
    }

    fn build_once_on_main_rank(&self, sim: &mut Simulation) {
        sim.add_startup_system(setup_time_series_output_system.after(make_output_dirs_system));
    }

    fn build_on_main_rank(&self, sim: &mut Simulation) {
        sim.add_startup_system(
            initialize_output_files_system::<T>
                .after(make_output_dirs_system)
                .after(setup_time_series_output_system),
        )
        .add_system_to_stage(OutputStages::Output, output_time_series_system::<T>);
    }

    fn build_everywhere(&self, sim: &mut Simulation) {
        // Add this here too, so we can request this even on systems running on non-main ranks without the crash.
        sim.add_event::<T>();
    }
}

fn make_time_series_dir(time_series_dir: &Path) {
    fs::create_dir_all(time_series_dir)
        .unwrap_or_else(|_| panic!("Failed to create time series dir: {time_series_dir:?}"));
}

fn setup_time_series_output_system(parameters: Res<OutputParameters>) {
    let time_series_dir = parameters.time_series_dir();
    make_time_series_dir(&time_series_dir);
}

fn initialize_output_files_system<T: TimeSeries>(
    parameters: Res<OutputParameters>,
    descriptor: NonSend<OutputDatasetDescriptor<T>>,
) where
    T: TimeSeries,
{
    let filename = &format!("{}.hdf5", descriptor.dataset_name());
    let time_series_dir = parameters.time_series_dir();
    let file = File::create(time_series_dir.join(filename))
        .expect("Failed to open time series output file");
    // Initialize empty datasets
    create_empty_dataset::<SimulationTime>(
        &file,
        &DatasetDescriptor::default_for::<SimulationTime>(),
    );
    create_empty_dataset::<T>(&file, &descriptor);
}

pub fn output_time_series_system<T: TimeSeries>(
    mut event_reader: EventReader<T>,
    time: Res<SimulationTime>,
    parameters: Res<OutputParameters>,
    descriptor: NonSend<OutputDatasetDescriptor<T>>,
) where
    T: TimeSeries,
{
    let path = get_time_series_filename::<T>(&parameters, &descriptor);
    let file = File::open_rw(path).expect("Failed to open time series output file");
    let time_dataset = file
        .dataset(TIME_DATASET_IDENTIFIER)
        .expect("Time dataset not available in file");
    let value_dataset = file
        .dataset(descriptor.dataset_name())
        .expect("Value dataset not available in file");
    for event in event_reader.iter() {
        append_value_to_dataset(&time_dataset, *time);
        append_value_to_dataset(&value_dataset, event.clone());
    }
}

fn append_value_to_dataset<T: ToDataset>(dataset: &Dataset, value: T) {
    let mut shape = dataset.shape();
    shape[0] += 1;
    dataset
        .resize(shape.clone())
        .expect("Failed to resize dataset");
    dataset
        .write_slice(&[value], [shape[0] - 1])
        .expect("Failed to write time to dataset");
}

fn get_time_series_filename<T: TimeSeries>(
    parameters: &OutputParameters,
    descriptor: &OutputDatasetDescriptor<T>,
) -> PathBuf {
    let time_series_dir = parameters.time_series_dir();
    time_series_dir.join(format!("{}.hdf5", descriptor.dataset_name()))
}
