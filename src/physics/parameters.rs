use serde::Deserialize;

use crate::units::Dimensionless;
use crate::units::Length;

#[derive(Deserialize)]
pub(super) struct Parameters {
    pub softening_length: Length,
    pub opening_angle: Dimensionless,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            softening_length: Length::zero(),
            opening_angle: Dimensionless::zero(),
        }
    }
}
