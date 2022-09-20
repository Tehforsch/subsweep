use serde::Deserialize;

use crate::named::Named;
use crate::units::Dimensionless;
use crate::units::Length;

#[derive(Clone, Deserialize, Named)]
#[name = "gravity"]
pub struct GravityParameters {
    pub softening_length: Length,
    pub opening_angle: Dimensionless,
}

impl Default for GravityParameters {
    fn default() -> Self {
        Self {
            softening_length: Length::zero(),
            opening_angle: Dimensionless::dimensionless(0.5),
        }
    }
}
