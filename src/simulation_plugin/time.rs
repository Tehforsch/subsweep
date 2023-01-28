use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use bevy::prelude::Resource;
use derive_more::From;
use hdf5::H5Type;

use crate::io::output::ToAttribute;
use crate::named::Named;

#[derive(H5Type, Clone, Copy, Deref, DerefMut, Named, Resource, From)]
#[repr(transparent)]
#[name = "time"]
pub struct Time(pub crate::units::Time);

impl ToAttribute for Time {
    type Output = crate::units::Time;

    fn to_value(&self) -> Self::Output {
        self.0
    }
}
