use bevy::prelude::Component;
use mpi::traits::Equivalence;

use crate::units::VecLength;

#[derive(Component, Debug, Clone, Equivalence)]
pub struct Position(pub VecLength);
