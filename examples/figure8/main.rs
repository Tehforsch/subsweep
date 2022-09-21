use std::path::Path;

use tenet::prelude::*;

fn main() {
    let mut sim = SimulationBuilder::mpi();
    sim.parameter_file_path(Path::new("examples/figure8/parameters.yml"))
        .headless(false)
        .verbosity(1)
        .build()
        .add_plugin(GravityPlugin)
        .run();
}
