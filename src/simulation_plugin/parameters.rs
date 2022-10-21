use serde::Deserialize;

use crate::named::Named;
use crate::units::Time;

/// General simulation parameters.
#[derive(Clone, Deserialize, Named, Default)]
#[name = "simulation"]
#[serde(deny_unknown_fields)]
pub struct SimulationParameters {
    /// If set to some value, the simulation will exit once the
    /// simulation time is larger or equal to this value.  If None,
    /// run indefinitely.
    #[serde(default)]
    pub final_time: Option<Time>,
}
