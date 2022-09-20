use std::path::Path;

use tenet::simulation_builder::SimulationBuilder;
use tenet::*;

fn main() {
    let mut sim = SimulationBuilder::mpi();
    sim.parameter_file_path(Path::new("examples/figure8/parameters.yml"))
        .headless(false)
        .build()
        .add_plugin(GravityPlugin)
        .run();
}
