use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct Parameters {
    pub show_quadtree: bool,
}
