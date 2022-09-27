use serde::Deserialize;

use crate::named::Named;

#[derive(Deserialize, Default, Named)]
#[name = "visualization"]
#[serde(deny_unknown_fields)]
pub struct VisualizationParameters {
    pub show_quadtree: bool,
}
