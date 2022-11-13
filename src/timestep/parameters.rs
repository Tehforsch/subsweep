use derive_custom::raxiom_parameters;

use crate::units::Time;

#[raxiom_parameters("timestep")]
pub struct TimestepParameters {
    #[serde(default = "default_num_levels")]
    pub num_levels: usize,
    pub max_timestep: Time,
}

fn default_num_levels() -> usize {
    1
}
