use bevy::prelude::Component;
use hdf5::H5Type;
use mpi::traits::Equivalence;

#[derive(H5Type, Component, Debug, Clone, Equivalence)]
#[repr(C)]
pub struct Mass(pub crate::units::Mass);
