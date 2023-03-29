use bevy::prelude::Component;
use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use derive_more::From;
use hdf5::H5Type;
use mpi::traits::Equivalence;

use crate::named::Named;
pub use crate::sweep::components::*;
use crate::units::VecLength;
use crate::units::VecVelocity;

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named)]
#[name = "position"]
#[repr(transparent)]
pub struct Position(pub VecLength);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named)]
#[repr(transparent)]
#[name = "mass"]
pub struct Mass(pub crate::units::Mass);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named)]
#[name = "velocity"]
#[repr(transparent)]
pub struct Velocity(pub VecVelocity);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Default, Named)]
#[repr(transparent)]
#[name = "density"]
pub struct Density(pub crate::units::Density);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Default, Named)]
#[name = "internal_energy"]
#[repr(transparent)]
pub struct InternalEnergy(pub crate::units::Energy);
