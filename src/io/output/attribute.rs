use std::marker::PhantomData;

use bevy_ecs::prelude::IntoSystemDescriptor;
use bevy_ecs::prelude::Res;
use bevy_ecs::prelude::ResMut;
use bevy_ecs::prelude::Resource;
use bevy_ecs::schedule::SystemDescriptor;
use hdf5::H5Type;

use super::plugin::IntoOutputSystem;
use super::OutputFile;
use crate::named::Named;

pub trait ToAttribute: Named + Resource {
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
    fn system() -> SystemDescriptor {
        write_attribute::<T>.into_descriptor()
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

// The poor man's procedural macro
#[macro_export]
macro_rules! impl_attribute {
    ($name: ident, $output: ty) => {
        impl ToAttribute for $name {
            type Output = $output;

            fn to_value(&self) -> Self::Output {
                self.0
            }
        }

        impl crate::io::input::attribute::FromAttribute for $name {
            fn from_value(val: <Self as ToAttribute>::Output) -> Self {
                Self(val)
            }
        }
    };
}
