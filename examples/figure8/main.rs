use std::path::Path;

use tenet::communication::BaseCommunicationPlugin;
use tenet::simulation::Simulation;
use tenet::*;

fn main() {
    let mut sim = Simulation::new();
    sim.add_parameters_from_file(Path::new("examples/figure8/parameters.yml"))
        .add_plugin(BaseCommunicationPlugin::new(1, 0))
        .add_plugin(InputPlugin)
        .add_plugin(SimulationPlugin { visualize: true })
        .add_plugin(GravityPlugin);
    sim.run();
}
