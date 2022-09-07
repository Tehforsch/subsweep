use std::marker::PhantomData;

use bevy::prelude::Plugin;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use hdf5::H5Type;

use super::add_output_system;
use super::OutputFile;

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

impl<T> Plugin for AttributePlugin<T>
where
    T: Clone + H5Type + Sync + Send + 'static,
{
    fn build(&self, app: &mut bevy::prelude::App) {
        let output_name = self.output_name.clone();
        add_output_system(app, move |res: Res<T>, file: ResMut<OutputFile>| {
            Self::write_attribute(&output_name, res, file)
        });
    }
}

impl<T> AttributePlugin<T>
where
    T: Clone + H5Type + Sync + Send + 'static,
{
    fn write_attribute(name: &str, res: Res<T>, file: ResMut<OutputFile>) {
        let f = file.f.as_ref().unwrap();
        let attr = f.new_attr::<T>().shape(()).create(name).unwrap();
        attr.write_scalar(&res.clone()).unwrap();
    }
}
