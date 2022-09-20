use serde::Deserialize;

use crate::units::Time;

#[derive(Clone, Deserialize)]
pub(super) struct Parameters {
    pub timestep: Time,
    pub final_time: Option<Time>,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            timestep: Time::seconds(1.0),
            final_time: None,
        }
    }
}
