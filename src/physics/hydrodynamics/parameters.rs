use serde::Deserialize;

use crate::named::Named;
use crate::units::Length;

#[derive(Deserialize, Named)]
#[name = "hydrodynamics"]
pub struct HydrodynamicsParameters {
    pub smoothing_length: Length,
}
