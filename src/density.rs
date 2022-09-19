use bevy::prelude::Component;
use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use derive_more::From;
use hdf5::H5Type;
use mpi::traits::Equivalence;

use crate::named::Named;

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From)]
#[repr(C)]
pub struct Density(pub crate::units::Density);

impl Named for Density {
    fn name() -> &'static str {
        "density"
    }
}
