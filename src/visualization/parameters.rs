use serde::Deserialize;

use crate::named::Named;

/// Parameters controlling the visualization. Only required if
/// headless is set to false
/// in the [SimulationBuilder](crate::prelude::SimulationBuilder).
#[derive(Deserialize, Default, Named)]
#[name = "visualization"]
#[serde(deny_unknown_fields)]
pub struct VisualizationParameters {
    #[serde(default)]
    pub show_quadtree: bool,
}
