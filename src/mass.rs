use bevy::prelude::Component;
use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use hdf5::H5Type;
use mpi::traits::Equivalence;

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut)]
#[repr(C)]
pub struct Mass(pub crate::units::Mass);
