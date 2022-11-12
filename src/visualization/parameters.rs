use derive_custom::raxiom_parameters;

use super::show_particles::ColorMap;

/// Parameters controlling the visualization. Only required if
/// headless is set to false
/// in the [SimulationBuilder](crate::prelude::SimulationBuilder).
#[raxiom_parameters("visualization")]
#[derive(Default)]
pub struct VisualizationParameters {
    #[serde(default)]
    pub show_quadtree: bool,
    #[serde(default)]
    pub show_particles: bool,
    #[serde(default)]
    pub color_map: ColorMap,
    #[serde(default)]
    pub show_halo_particles: bool,
    #[serde(default = "default_show_box_size")]
    pub show_box_size: bool,
}

fn default_show_box_size() -> bool {
    true
}
