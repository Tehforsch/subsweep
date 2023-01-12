use bevy::prelude::Commands;
use bevy::prelude::Res;

use crate::grid::init_cartesian_grid_system;
use crate::parameters::SimulationBox;
use crate::parameters::SweepParameters;
use crate::simulation::Simulation;
use crate::stages::SimulationStagesPlugin;
use crate::sweep::parameters::DirectionsSpecification;
use crate::sweep::SweepPlugin;
use crate::units::Density;
use crate::units::Dimensionless;
use crate::units::Length;
use crate::units::MVec;

#[test]
fn simple_sweep() {
    let num_cells = 10;
    let cell_size = Length::meters(0.1);
    let simulation_box = SimulationBox::cube_from_side_length(cell_size * num_cells as f64);
    let density = Density::zero();
    let mut sim = Simulation::test();
    sim.add_parameters_explicitly(simulation_box)
        .add_parameters_explicitly(SweepParameters {
            directions: DirectionsSpecification::Explicit(vec![
                MVec::ONE * Dimensionless::dimensionless(1.0),
            ]),
        })
        .add_startup_system(move |commands: Commands, box_size: Res<SimulationBox>| {
            init_cartesian_grid_system(commands, box_size, cell_size, density)
        })
        .add_plugin(SimulationStagesPlugin)
        .add_plugin(SweepPlugin);
    sim.update();
}
