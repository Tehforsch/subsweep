use std::marker::PhantomData;

use bevy::prelude::Component;
use bevy::prelude::ParallelSystemDescriptorCoercion;
use bevy::prelude::Plugin;
use bevy::prelude::Query;
use bevy::prelude::ResMut;
use hdf5::H5Type;

use super::close_file_system;
use super::open_file_system;
use super::output_setup;
use super::timer::Timer;
use super::OutputFile;
use super::OutputStages;
use super::OutputSystemsAmbiguitySet;
use crate::plugin_utils::run_once;

pub struct DatasetPlugin<T> {
    _marker: PhantomData<T>,
    output_name: String,
}

impl<T> DatasetPlugin<T> {
    pub fn new(name: &str) -> Self {
        Self {
            _marker: PhantomData::default(),
            output_name: name.into(),
        }
    }
}

impl<T: Clone + H5Type + Component + Sync + Send + 'static> Plugin for DatasetPlugin<T> {
    fn build(&self, app: &mut bevy::prelude::App) {
        run_once("output_plugin", app, |app| output_setup(app));
        let output_name = self.output_name.clone();
        app.add_system_to_stage(
            OutputStages::Output,
            (move |query: Query<&T>, file: ResMut<OutputFile>| {
                Self::write_dataset(&output_name, query, file)
            })
            .after(open_file_system)
            .before(close_file_system)
            .in_ambiguity_set(OutputSystemsAmbiguitySet)
            .with_run_criteria(Timer::run_criterion),
        );
    }
}

impl<T: Sync + Send + 'static + Clone + H5Type + Component> DatasetPlugin<T> {
    fn write_dataset(name: &str, query: Query<&T>, file: ResMut<OutputFile>) {
        let f = file.f.as_ref().unwrap();
        let data: Vec<T> = query.iter().cloned().collect();
        f.new_dataset_builder()
            .with_data(&data)
            .create(name)
            .expect("Failed to write dataset");
    }
}
