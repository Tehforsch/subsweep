use bevy::prelude::Component;
use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use derive_custom::Named;
use derive_more::From;
use hdf5::H5Type;
use mpi::traits::Equivalence;

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named)]
#[name = "source"]
#[repr(transparent)]
pub struct AbsorptionRate(pub crate::units::PhotonFlux);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named)]
#[name = "ionized_hydrogen_fraction"]
#[repr(transparent)]
pub struct HydrogenIonizationFraction(pub crate::units::Dimensionless);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named)]
#[name = "source"]
#[repr(transparent)]
pub struct Source(pub crate::units::SourceRate);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named)]
#[name = "flux"]
#[repr(transparent)]
pub struct Flux(pub crate::units::PhotonFlux);
