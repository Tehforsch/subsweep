use std::marker::PhantomData;

use bevy::prelude::Plugin;
use bevy::prelude::Res;
use bevy::prelude::ResMut;

use super::add_output_system;
use super::attribute::Attribute;
use super::OutputFile;

pub struct AttributePlugin<T> {
    _marker: PhantomData<T>,
    name: String,
}

impl<T> AttributePlugin<T> {
    pub fn new(name: &str) -> Self {
        Self {
            _marker: PhantomData::default(),
            name: name.into(),
        }
    }
}

impl<T> Plugin for AttributePlugin<T>
where
    T: Attribute + Sync + Send + 'static,
{
    fn build(&self, app: &mut bevy::prelude::App) {
        let output_name = self.name.clone();
        add_output_system(
            app,
            &self.name,
            move |res: Res<T>, file: ResMut<OutputFile>| {
                Self::write_attribute(&output_name, res, file)
            },
        );
    }
}

impl<T> AttributePlugin<T>
where
    T: Attribute + Sync + Send + 'static,
{
    fn write_attribute(name: &str, res: Res<T>, file: ResMut<OutputFile>) {
        let f = file.f.as_ref().unwrap();
        let attr = f.new_attr::<T::Output>().shape(()).create(name).unwrap();
        attr.write_scalar(&res.to_value()).unwrap();
    }
}
