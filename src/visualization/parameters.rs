use serde::Deserialize;

use crate::named::Named;
use crate::units::Length;

#[derive(Deserialize, Default, Named)]
#[name = "visualization"]
pub struct VisualizationParameters {
    pub show_quadtree: bool,
    pub camera_zoom: Length,
}
