use std::marker::PhantomData;

use bevy::prelude::Component;
use bevy::prelude::Plugin;
use bevy::prelude::Query;
use bevy::prelude::ResMut;
use hdf5::H5Type;

use super::add_output_system;
use super::OutputFile;
use crate::named::Named;

pub struct DatasetPlugin<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for DatasetPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData::default(),
        }
    }
}

impl<T: Named + Clone + H5Type + Component + Sync + Send + 'static> Plugin for DatasetPlugin<T> {
    fn build(&self, app: &mut bevy::prelude::App) {
        add_output_system::<T, _>(app, move |query: Query<&T>, file: ResMut<OutputFile>| {
            Self::write_dataset(query, file)
        })
    }
}

impl<T: Named + Sync + Send + 'static + Clone + H5Type + Component> DatasetPlugin<T> {
    fn write_dataset(query: Query<&T>, file: ResMut<OutputFile>) {
        let f = file.f.as_ref().unwrap();
        let data: Vec<T> = query.iter().cloned().collect();
        f.new_dataset_builder()
            .with_data(&data)
            .create(T::name())
            .expect("Failed to write dataset");
    }
}
