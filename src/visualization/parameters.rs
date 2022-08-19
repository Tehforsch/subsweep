use serde::Deserialize;

#[derive(Deserialize, Default)]
pub(super) struct Parameters {
    pub show_quadtree: bool,
}
