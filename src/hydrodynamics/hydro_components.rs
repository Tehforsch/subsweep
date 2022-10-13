use bevy::prelude::Component;
use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use derive_more::From;
use hdf5::H5Type;
use mpi::traits::Equivalence;

use crate::named::Named;

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Default, Named)]
#[repr(transparent)]
#[name = "density"]
pub struct Density(pub crate::units::Density);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Default, Named)]
#[name = "pressure"]
#[repr(transparent)]
pub struct Pressure(pub crate::units::Pressure);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Default, Named)]
#[name = "smoothing_length"]
#[repr(transparent)]
pub struct SmoothingLength(pub crate::units::Length);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Default, Named)]
#[name = "internal_energy"]
#[repr(transparent)]
pub struct InternalEnergy(pub crate::units::Energy);
