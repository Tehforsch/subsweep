use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use bevy::prelude::Resource;
use derive_more::From;
use hdf5::H5Type;

use crate::impl_attribute;
use crate::io::output::ToAttribute;
use crate::named::Named;

#[derive(H5Type, Clone, Copy, Deref, DerefMut, Named, Resource, From)]
#[repr(transparent)]
#[name = "time"]
pub struct SimulationTime(pub crate::units::Time);

impl_attribute!(SimulationTime, crate::units::Time);
