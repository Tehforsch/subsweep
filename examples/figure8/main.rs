use std::path::Path;

use tenet::communication::BaseCommunicationPlugin;
use tenet::plugin_utils::Simulation;
use tenet::*;

fn main() {
    let mut app = Simulation::new();
    add_parameter_file_contents(&mut app, Path::new("examples/figure8/parameters.yml"));
    app.add_plugin(BaseCommunicationPlugin::new(1, 0))
        .add_tenet_plugin(InputPlugin)
        .add_tenet_plugin(SimulationPlugin { visualize: true })
        .add_tenet_plugin(GravityPlugin);
    app.run();
}
