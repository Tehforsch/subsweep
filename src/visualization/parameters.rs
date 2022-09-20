use serde::Deserialize;

use crate::units::Length;

#[derive(Deserialize, Default)]
pub struct VisualizationParameters {
    pub show_quadtree: bool,
    pub camera_zoom: Length,
}
