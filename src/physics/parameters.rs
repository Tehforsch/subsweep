use serde::Deserialize;

use crate::named::Named;
use crate::units::Time;

#[derive(Clone, Deserialize, Named)]
#[name = "simulation"]
#[serde(deny_unknown_fields)]
pub(super) struct SimulationParameters {
    pub timestep: Time,
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
