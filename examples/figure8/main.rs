use raxiom::prelude::*;

fn main() {
    let mut sim = SimulationBuilder::new();
    sim.parameters_from_relative_path(file!(), "parameters.yml")
        .headless(false)
        .update_from_command_line_options()
        .build()
        .add_plugin(GravityPlugin)
        .run();
}
