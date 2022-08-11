use bevy::prelude::Component;
use mpi::traits::Equivalence;

use crate::units::vec2::Length;

#[derive(Component, Debug, Clone, Equivalence)]
pub struct Position(pub Length);
