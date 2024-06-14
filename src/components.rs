use bevy_ecs::prelude::Component;
use derive_more::Deref;
use derive_more::DerefMut;
use derive_more::From;
use hdf5::H5Type;
use mpi::traits::Equivalence;

use crate::named::Named;
use crate::prelude::Float;
use crate::units;
use crate::units::Time;
use crate::units::VecLength;

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named)]
#[name = "position"]
#[repr(transparent)]
pub struct Position(pub VecLength);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Default, Named)]
#[repr(transparent)]
#[name = "density"]
pub struct Density(pub crate::units::Density);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named)]
#[repr(transparent)]
#[name = "mass"]
pub struct Mass(pub crate::units::Mass);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named)]
#[name = "ionized_hydrogen_fraction"]
#[repr(transparent)]
pub struct IonizedHydrogenFraction(pub crate::units::Dimensionless);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named)]
#[name = "delta_ionized_hydrogen_fraction"]
#[repr(transparent)]
pub struct DeltaIonizedHydrogenFraction(pub crate::units::Dimensionless);

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
#[name = "photoionization_rate"]
#[repr(transparent)]
pub struct PhotoionizationRate(pub crate::units::Rate);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named, Default)]
#[name = "recombination_rate"]
#[repr(transparent)]
pub struct RecombinationRate(pub crate::units::Rate);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named, Default)]
#[name = "collisional_ionization_rate"]
#[repr(transparent)]
pub struct CollisionalIonizationRate(pub crate::units::Rate);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Named, Default)]
#[name = "heating_rate"]
#[repr(transparent)]
pub struct HeatingRate(pub units::HeatingRate);

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

#[macro_export]
macro_rules! impl_to_dataset {
    ($name: ty, $dim: ty, $is_static: expr) => {
        impl $crate::io::to_dataset::ToDataset for $name {
            fn dimension() -> crate::units::Dimension {
                <$dim>::dimension()
            }

            fn convert_base_units(self, factor: f64) -> Self {
                Self(self.0 * factor)
            }

            fn is_static() -> bool {
                $is_static
            }
        }
    };
}

// Static quantities
impl_to_dataset!(Position, units::Length, true);
impl_to_dataset!(Density, units::Density, true);
impl_to_dataset!(Source, units::SourceRate, true);
impl_to_dataset!(Mass, units::Mass, true);

// Dynamic quantities
impl_to_dataset!(IonizedHydrogenFraction, units::Dimensionless, false);
impl_to_dataset!(DeltaIonizedHydrogenFraction, units::Dimensionless, false);
impl_to_dataset!(Temperature, units::Temperature, false);
impl_to_dataset!(PhotonRate, units::SourceRate, false);
impl_to_dataset!(PhotoionizationRate, units::Rate, false);
impl_to_dataset!(RecombinationRate, units::Rate, false);
impl_to_dataset!(CollisionalIonizationRate, units::Rate, false);
impl_to_dataset!(HeatingRate, units::HeatingRate, false);
impl_to_dataset!(Timestep, units::Time, false);
impl_to_dataset!(IonizationTime, units::Time, false);
