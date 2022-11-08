use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use derive_custom::raxiom_parameters;
use derive_more::From;
use derive_more::Into;

use crate::domain::Extent;
use crate::units::Length;
use crate::units::Time;

/// General simulation parameters.
#[raxiom_parameters("simulation")]
pub struct SimulationParameters {
    /// If set to some value, the simulation will exit once the
    /// simulation time is larger or equal to this value.  If None,
    /// run indefinitely.
    #[serde(default)]
    pub final_time: Option<Time>,
}

/// The box size of the simulation. Periodic boundary conditions apply
/// beyond this box, meaning that the positions of particles outside
/// of this bax are wrapped back into it.
#[raxiom_parameters("box_size")]
#[derive(From, Into, Deref, DerefMut)]
pub struct BoxSize(Extent);

impl BoxSize {
    pub fn cube_from_side_length(side_length: Length) -> Self {
        Self(Extent::cube_from_side_length(side_length))
    }
}
