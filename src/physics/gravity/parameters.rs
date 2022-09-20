use serde::Deserialize;

use crate::units::Dimensionless;
use crate::units::Length;

#[derive(Clone, Deserialize)]
pub struct Parameters {
    pub softening_length: Length,
    pub opening_angle: Dimensionless,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            softening_length: Length::zero(),
            opening_angle: Dimensionless::dimensionless(0.5),
        }
    }
}
