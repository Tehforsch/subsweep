use std::marker::PhantomData;

use bevy::prelude::Component;
use bevy::prelude::Plugin;
use bevy::prelude::Query;
use bevy::prelude::ResMut;
use hdf5::H5Type;

use super::add_output_system;
use super::OutputFile;

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
        let output_name = self.output_name.clone();
        add_output_system(app, move |query: Query<&T>, file: ResMut<OutputFile>| {
            Self::write_dataset(&output_name, query, file)
        })
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
