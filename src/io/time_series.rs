use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::marker::PhantomData;
use std::path::Path;
use std::path::PathBuf;

use bevy_ecs::prelude::EventReader;
use bevy_ecs::prelude::IntoSystemDescriptor;
use bevy_ecs::prelude::NonSend;
use bevy_ecs::prelude::Res;
use serde::Serialize;

use super::output::make_output_dirs_system;
use super::DatasetDescriptor;
use super::OutputDatasetDescriptor;
use crate::named::Named;
use crate::parameters::Cosmology;
use crate::parameters::OutputParameters;
use crate::prelude::Stages;
use crate::simulation::Simulation;
use crate::simulation::SubsweepPlugin;
use crate::simulation_plugin::SimulationTime;
use crate::time_spec::TimeSpec;

pub trait TimeSeries: 'static + Sync + Send + Clone + Serialize {}

impl<T> TimeSeries for T where T: 'static + Sync + Send + Clone + Serialize {}

#[derive(Serialize)]
struct Entry<T> {
    time: TimeSpec,
    val: T,
}

#[derive(Named)]
pub struct TimeSeriesPlugin<T: TimeSeries> {
    descriptor: OutputDatasetDescriptor<T>,
}

impl<T: Named + TimeSeries> Default for TimeSeriesPlugin<T> {
    fn default() -> Self {
        Self {
            descriptor: OutputDatasetDescriptor {
                _marker: PhantomData,
                descriptor: DatasetDescriptor::default_for::<T>(),
            },
        }
    }
}

impl<T: TimeSeries> SubsweepPlugin for TimeSeriesPlugin<T> {
    fn should_build(&self, sim: &Simulation) -> bool {
        sim.write_output
    }

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
        .add_system_to_stage(Stages::Output, output_time_series_system::<T>);
    }

    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.insert_non_send_resource::<OutputDatasetDescriptor<T>>(
            OutputDatasetDescriptor::<T>::new(self.descriptor.descriptor.clone()),
        );
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
    let filename = get_time_series_filename(&parameters, &descriptor);
    File::create(filename).expect("Failed to open time series output file");
}

pub fn output_time_series_system<T: TimeSeries>(
    mut event_reader: EventReader<T>,
    time: Res<SimulationTime>,
    parameters: Res<OutputParameters>,
    cosmology: Res<Cosmology>,
    descriptor: NonSend<OutputDatasetDescriptor<T>>,
) where
    T: TimeSeries,
{
    let path = get_time_series_filename::<T>(&parameters, &descriptor);
    let entries: Vec<_> = event_reader
        .iter()
        .map(|ev| Entry {
            time: TimeSpec::new(**time, &cosmology),
            val: ev.clone(),
        })
        .collect();
    if entries.len() > 0 {
        let f = OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap_or_else(|e| panic!("Failed to open time series file. {}", e));
        serde_yaml::to_writer(&f, &entries)
            .unwrap_or_else(|e| panic!("Failed to write to time series file: {}", e));
    }
}

fn get_time_series_filename<T: TimeSeries>(
    parameters: &OutputParameters,
    descriptor: &OutputDatasetDescriptor<T>,
) -> PathBuf {
    let time_series_dir = parameters.time_series_dir();
    time_series_dir.join(format!("{}.yml", descriptor.dataset_name()))
}
