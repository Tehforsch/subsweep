use bevy::prelude::Component;
use mpi::traits::Equivalence;

use crate::units::VecVelocity;

#[derive(Component, Debug, Clone, Equivalence)]
pub struct Velocity(pub VecVelocity);
