use std::path::Path;

use bevy::prelude::*;
use tenet::communication::BaseCommunicationPlugin;
use tenet::*;

fn main() {
    let mut app = App::new();
    add_parameter_file_contents(&mut app, Path::new("examples/figure8/parameters.yml"));
    app.add_plugin(BaseCommunicationPlugin::new(1, 0))
        .add_plugin(InputPlugin)
        .add_plugin(SimulationPlugin { visualize: true })
        .add_plugin(GravityPlugin);
    app.run();
}
