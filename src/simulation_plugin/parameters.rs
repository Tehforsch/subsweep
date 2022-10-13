use serde::Deserialize;

use crate::named::Named;
use crate::units::Time;

/// General simulation parameters.
#[derive(Clone, Deserialize, Named)]
#[name = "simulation"]
#[serde(deny_unknown_fields)]
pub struct SimulationParameters {
    /// The timestep of the simulation. Will soon become
    /// a maximum/minimum timestep setting instead
    pub timestep: Time,
    /// If set to some value, the simulation will exit once the
    /// simulation time is larger or equal to this value.  If None,
    /// run indefinitely.
    #[serde(default)]
    pub final_time: Option<Time>,
}

impl Default for SimulationParameters {
    fn default() -> Self {
        Self {
            timestep: Time::seconds(1.0),
            final_time: None,
        }
    }
}
