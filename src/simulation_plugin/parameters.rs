use derive_custom::raxiom_parameters;

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
