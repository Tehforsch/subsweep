use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use derive_more::From;
use derive_more::Into;
use serde::Deserialize;
use serde::Serialize;

use crate::domain::Extent;
use crate::named::Named;
use crate::units::Length;
use crate::units::Time;

/// General simulation parameters.
#[derive(Clone, Serialize, Deserialize, Named)]
#[name = "simulation"]
#[serde(deny_unknown_fields)]
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
#[derive(Clone, Serialize, Deserialize, From, Into, Named, Deref, DerefMut)]
#[name = "box_size"]
pub struct BoxSize(Extent);

impl BoxSize {
    pub fn cube_from_side_length(side_length: Length) -> Self {
        Self(Extent::cube_from_side_length(side_length))
    }
}
