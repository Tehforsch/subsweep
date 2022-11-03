use serde::Deserialize;

use crate::named::Named;

/// Parameters controlling the visualization. Only required if
/// headless is set to false
/// in the [SimulationBuilder](crate::prelude::SimulationBuilder).
#[derive(Clone, Deserialize, Default, Named)]
#[name = "visualization"]
#[serde(deny_unknown_fields)]
pub struct VisualizationParameters {
    #[serde(default)]
    pub show_quadtree: bool,
    #[serde(default)]
    pub show_particles: bool,
    #[serde(default)]
    pub show_halo_particles: bool,
}
