use serde::Deserialize;

use crate::units::f32::Length;

#[derive(Deserialize)]
pub(super) struct Parameters {
    pub softening_length: Length,
}
