use serde::Deserialize;

use crate::named::Named;
use crate::units::Time;

#[derive(Clone, Deserialize, Named)]
#[name = "timestep"]
#[serde(deny_unknown_fields)]
pub struct TimestepParameters {
    #[serde(default = "default_num_levels")]
    pub num_levels: usize,
    pub max_timestep: Time,
}

fn default_num_levels() -> usize {
    1
}
