use serde::Deserialize;

use crate::named::Named;

#[derive(Deserialize, Default, Named)]
#[name = "visualization"]
pub struct VisualizationParameters {
    pub show_quadtree: bool,
}
