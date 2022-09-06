use std::marker::PhantomData;

use bevy::prelude::ParallelSystemDescriptorCoercion;
use bevy::prelude::Plugin;
use bevy::prelude::Res;
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

pub struct AttributePlugin<T> {
    _marker: PhantomData<T>,
    output_name: String,
}

impl<T> AttributePlugin<T> {
    pub fn new(name: &str) -> Self {
        Self {
            _marker: PhantomData::default(),
            output_name: name.into(),
        }
    }
}

impl<T: Clone + H5Type + Sync + Send + 'static> Plugin for AttributePlugin<T> {
    fn build(&self, app: &mut bevy::prelude::App) {
        run_once("output_plugin", app, |app| output_setup(app));
        let output_name = self.output_name.clone();
        app.add_system_to_stage(
            OutputStages::Output,
            (move |res: Res<T>, file: ResMut<OutputFile>| {
                Self::write_attribute(&output_name, res, file)
            })
            .after(open_file_system)
            .before(close_file_system)
            .in_ambiguity_set(OutputSystemsAmbiguitySet)
            .with_run_criteria(Timer::run_criterion),
        );
    }
}

impl<T: Sync + Send + 'static + Clone + H5Type> AttributePlugin<T> {
    fn write_attribute(name: &str, res: Res<T>, file: ResMut<OutputFile>) {
        let f = file.f.as_ref().unwrap();
        let attr = f.new_attr::<T>().shape([1]).create(name).unwrap();
        attr.write(&[res.clone()]).unwrap();
    }
}
