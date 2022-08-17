use bevy::prelude::Component;
use mpi::traits::Equivalence;

use crate::units::f32;

#[derive(Component, Debug, Clone, Equivalence)]
pub struct Mass(pub f32::Mass);
