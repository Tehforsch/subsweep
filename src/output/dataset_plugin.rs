use std::marker::PhantomData;

use bevy::prelude::Component;
use bevy::prelude::Plugin;
use bevy::prelude::Query;
use bevy::prelude::ResMut;
use bevy::prelude::With;
use hdf5::H5Type;

use super::add_output_system;
use super::to_dataset::ToDataset;
use super::OutputFile;
use crate::named::Named;
use crate::physics::LocalParticle;

pub struct DatasetOutputPlugin<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for DatasetOutputPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData::default(),
        }
    }
}

impl<T: ToDataset + Named + Clone + Component + Sync + Send + 'static> Plugin
    for DatasetOutputPlugin<T>
{
    fn build(&self, app: &mut bevy::prelude::App) {
        add_output_system::<T, _>(app, Self::write_dataset);
    }
}

impl<T: ToDataset + Named + Sync + Send + 'static + Clone + H5Type + Component>
    DatasetOutputPlugin<T>
{
    fn write_dataset(query: Query<&T, With<LocalParticle>>, file: ResMut<OutputFile>) {
        let f = file.f.as_ref().unwrap();
        let data: Vec<T> = query.iter().cloned().collect();
        let dataset = f
            .new_dataset_builder()
            .with_data(&data)
            .create(T::name())
            .expect("Failed to write dataset");
        let attr = dataset
            .new_attr::<f64>()
            .shape(())
            .create("scale_factor")
            .unwrap();
        let scale_factor = T::dimension().base_conversion_factor();
        attr.write_scalar(&scale_factor).unwrap();
    }
}
