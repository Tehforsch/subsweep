mod cartesian;
mod cell;

pub use cartesian::init_cartesian_grid_system;
pub use cell::Cell;
pub use cell::Neighbour;
pub use cell::NeighbourKind;
use derive_custom::Named;

use crate::simulation::RaxiomPlugin;

#[derive(Named)]
struct GridPlugin {}

impl RaxiomPlugin for GridPlugin {
    fn build_everywhere(&self, _sim: &mut crate::simulation::Simulation) {}
}
