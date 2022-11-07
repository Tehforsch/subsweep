use serde::Deserialize;
use serde::Serialize;

use super::show_particles::ColorMap;
use crate::parameter_plugin::parameter_section;

/// Parameters controlling the visualization. Only required if
/// headless is set to false
/// in the [SimulationBuilder](crate::prelude::SimulationBuilder).
#[parameter_section("visualization")]
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
}
