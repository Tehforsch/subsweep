use serde::Deserialize;

use crate::units::Dimensionless;
use crate::units::Length;
use crate::units::Time;

#[derive(Clone, Deserialize)]
pub(super) struct Parameters {
    pub softening_length: Length,
    pub opening_angle: Dimensionless,
    pub timestep: Time,
    pub final_time: Option<Time>,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            softening_length: Length::zero(),
            opening_angle: Dimensionless::dimensionless(0.5),
            timestep: Time::second(1.0),
            final_time: None,
        }
    }
}
