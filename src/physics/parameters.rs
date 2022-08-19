use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct Parameters {
    softening: f32,
}
