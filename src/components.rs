use bevy_ecs::prelude::Component;
use derive_more::Deref;
use derive_more::DerefMut;
use derive_more::From;
use diman::Quotient;
use hdf5::H5Type;
use mpi::traits::Equivalence;

use crate::named::Named;
use crate::prelude::Float;
use crate::units::EnergyDensity;
use crate::units::Time;
use crate::units::VecLength;

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named)]
#[name = "position"]
#[repr(transparent)]
pub struct Position(pub VecLength);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named)]
#[repr(transparent)]
#[name = "mass"]
pub struct Mass(pub crate::units::Mass);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Default, Named)]
#[repr(transparent)]
#[name = "density"]
pub struct Density(pub crate::units::Density);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named)]
#[name = "ionized_hydrogen_fraction"]
#[repr(transparent)]
pub struct IonizedHydrogenFraction(pub crate::units::Dimensionless);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named)]
#[name = "temperature"]
#[repr(transparent)]
pub struct Temperature(pub crate::units::Temperature);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named)]
#[name = "source"]
#[repr(transparent)]
pub struct Source(pub crate::units::SourceRate);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named)]
#[name = "photon_rate"]
#[repr(transparent)]
pub struct PhotonRate(pub crate::units::PhotonRate);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named, Default)]
#[name = "heating_rate"]
#[repr(transparent)]
pub struct HeatingRate(pub Quotient<EnergyDensity, Time>);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named, Default)]
#[name = "timestep"]
#[repr(transparent)]
pub struct Timestep(pub Time);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named)]
#[name = "ionization_time"]
#[repr(transparent)]
pub struct IonizationTime(pub Time);

impl Default for IonizationTime {
    fn default() -> Self {
        IonizationTime(Time::new_unchecked(Float::INFINITY))
    }
}
