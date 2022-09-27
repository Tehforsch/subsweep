use serde::Deserialize;

use crate::named::Named;
use crate::units::Dimensionless;
use crate::units::Length;

#[derive(Clone, Deserialize, Named)]
#[name = "gravity"]
#[serde(deny_unknown_fields)]
pub struct GravityParameters {
    #[serde(default)]
    pub softening_length: Length,
    #[serde(default)]
    pub opening_angle: Dimensionless,
}
