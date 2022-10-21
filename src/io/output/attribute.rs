use std::marker::PhantomData;

use bevy::ecs::schedule::ParallelSystemDescriptor;
use bevy::prelude::ParallelSystemDescriptorCoercion;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use hdf5::H5Type;

use super::plugin::IntoOutputSystem;
use super::OutputFile;
use super::OutputSystemsAmbiguitySet;
use crate::named::Named;

pub trait ToAttribute: Named + Sync + Send + 'static {
    type Output: H5Type;
    fn to_value(&self) -> Self::Output;
}

pub struct Attribute<T> {
    _marker: PhantomData<T>,
}

impl<T: Named> Named for Attribute<T> {
    fn name() -> &'static str {
        T::name()
    }
}

impl<T: ToAttribute> IntoOutputSystem for Attribute<T> {
    fn system() -> ParallelSystemDescriptor {
        write_attribute::<T>.in_ambiguity_set(OutputSystemsAmbiguitySet)
    }
}

fn write_attribute<T: ToAttribute>(res: Res<T>, file: ResMut<OutputFile>) {
    let f = file.f.as_ref().unwrap();
    let attr = f
        .new_attr::<T::Output>()
        .shape(())
        .create(T::name())
        .unwrap();
    attr.write_scalar(&res.to_value()).unwrap();
}
