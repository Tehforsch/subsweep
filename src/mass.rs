use bevy::prelude::Component;
use mpi::traits::Equivalence;

#[derive(Component, Debug, Clone, Equivalence)]
pub struct Mass(pub crate::units::Mass);
