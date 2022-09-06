use bevy::prelude::Component;
use hdf5::H5Type;
use mpi::traits::Equivalence;

use crate::units::VecLength;

#[derive(H5Type, Component, Debug, Clone, Equivalence)]
#[repr(C)]
pub struct Position(pub VecLength);
