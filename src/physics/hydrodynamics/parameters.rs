use serde::Deserialize;

use crate::named::Named;
use crate::units::Length;

#[derive(Deserialize, Named)]
#[name = "hydrodynamics"]
#[serde(deny_unknown_fields)]
pub struct HydrodynamicsParameters {
    pub smoothing_length: Length,
}
