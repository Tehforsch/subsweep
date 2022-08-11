use bevy::prelude::Component;
use mpi::traits::Equivalence;

use crate::units::vec2;

#[derive(Component, Debug, Clone, Equivalence)]
pub struct Velocity(pub vec2::Velocity);
