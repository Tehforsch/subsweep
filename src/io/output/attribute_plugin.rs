use std::marker::PhantomData;

use bevy::ecs::schedule::ParallelSystemDescriptor;
use bevy::prelude::ParallelSystemDescriptorCoercion;
use bevy::prelude::Res;
use bevy::prelude::ResMut;

use super::attribute::Attribute;
use super::plugin::IntoOutputSystem;
use super::OutputFile;
use super::OutputSystemsAmbiguitySet;
use crate::named::Named;

#[derive(Named)]
pub struct AttributeOutputPlugin<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for AttributeOutputPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData::default(),
        }
    }
}

impl<T: Attribute + Sync + Send + 'static> IntoOutputSystem for AttributeOutputPlugin<T> {
    fn system() -> ParallelSystemDescriptor {
        Self::write_attribute.in_ambiguity_set(OutputSystemsAmbiguitySet)
    }
}

impl<T> AttributeOutputPlugin<T>
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
