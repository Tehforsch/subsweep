use derive_custom::raxiom_parameters;

use super::show_particles::ColorMap;
use crate::parameters::SimulationBox;
use crate::units::Length;
use crate::units::VecLength;

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
    #[serde(default)]
    pub slice: SliceSpecification,
}

fn default_show_box_size() -> bool {
    true
}

#[raxiom_parameters]
#[derive(Default)]
pub struct SliceSpecification(Option<(AxisSpecification, Length)>);

#[raxiom_parameters]
#[derive(Default)]
pub enum AxisSpecification {
    #[default]
    X,
    Y,
    Z,
}

impl AxisSpecification {
    #[cfg(feature = "3d")]
    fn to_vec(&self) -> crate::units::VecDimensionless {
        match self {
            AxisSpecification::X => crate::units::VecDimensionless::dimensionless(1.0, 0.0, 0.0),
            AxisSpecification::Y => crate::units::VecDimensionless::dimensionless(0.0, 1.0, 0.0),
            AxisSpecification::Z => crate::units::VecDimensionless::dimensionless(0.0, 0.0, 1.0),
        }
    }
}

impl SliceSpecification {
    #[cfg(not(feature = "2d"))]
    pub fn contains(&self, pos: VecLength, box_size: &SimulationBox) -> bool {
        let dist_to_center = pos - box_size.center();
        match &self.0 {
            Some((axis, thickness)) => dist_to_center.dot(axis.to_vec()).abs() < *thickness,
            None => true,
        }
    }

    #[cfg(feature = "2d")]
    pub fn contains(&self, _pos: VecLength, _box_size: &SimulationBox) -> bool {
        true
    }
}
