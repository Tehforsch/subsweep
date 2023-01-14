#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use bevy::prelude::*;
use raxiom::components;
use raxiom::components::Position;
use raxiom::grid::init_cartesian_grid_system;
use raxiom::prelude::*;
use raxiom::units::Density;
use raxiom::units::Dimensionless;
use raxiom::units::Length;

#[raxiom_parameters("sweep_postprocess")]
struct Parameters {
    cell_size: Length,
    density: Density,
}

fn main() {
    let mut sim = SimulationBuilder::new();
    let mut sim = sim
        .parameters_from_relative_path(file!(), "parameters.yml")
        .headless(false)
        .write_output(false)
        .read_initial_conditions(false)
        .update_from_command_line_options()
        .build();
    let parameters = sim
        .add_parameter_type_and_get_result::<Parameters>()
        .clone();
    sim.add_startup_system(move |commands: Commands, box_size: Res<SimulationBox>| {
        init_cartesian_grid_system(commands, box_size, parameters.cell_size)
    })
    .add_startup_system_to_stage(
        SimulationStartupStages::InsertDerivedComponents,
        initialize_sweep_components_system,
    )
    .add_plugin(SweepPlugin)
    .run();
}

fn initialize_sweep_components_system(
    mut commands: Commands,
    particles: Particles<(Entity, &Position)>,
    parameters: Res<Parameters>,
) {
    for (entity, _) in particles.iter() {
        commands.entity(entity).insert((
            components::Density(parameters.density),
            components::HydrogenAbundance(Dimensionless::zero()),
        ));
    }
}
