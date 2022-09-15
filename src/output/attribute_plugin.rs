use std::marker::PhantomData;

use bevy::prelude::Plugin;
use bevy::prelude::Res;
use bevy::prelude::ResMut;

use super::add_output_system;
use super::attribute::Attribute;
use super::OutputFile;

pub struct AttributePlugin<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for AttributePlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData::default(),
        }
    }
}

impl<T> Plugin for AttributePlugin<T>
where
    T: Attribute + Sync + Send + 'static,
{
    fn build(&self, app: &mut bevy::prelude::App) {
        add_output_system::<T, _>(app, move |res: Res<T>, file: ResMut<OutputFile>| {
            Self::write_attribute(res, file)
        });
    }
}

impl<T> AttributePlugin<T>
where
    T: Attribute + Sync + Send + 'static,
{
    fn write_attribute(res: Res<T>, file: ResMut<OutputFile>) {
        let f = file.f.as_ref().unwrap();
        let attr = f
            .new_attr::<T::Output>()
            .shape(())
            .create(T::name())
            .unwrap();
        attr.write_scalar(&res.to_value()).unwrap();
    }
}
