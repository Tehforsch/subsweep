use bevy::prelude::Component;
use hdf5::H5Type;
use mpi::traits::Equivalence;

use crate::units::VecVelocity;

#[derive(H5Type, Component, Debug, Clone, Equivalence)]
#[repr(C)]
pub struct Velocity(pub VecVelocity);
