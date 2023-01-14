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
use raxiom::units::Time;

#[raxiom_parameters("sweep_postprocess")]
struct Parameters {
    cell_size: Length,
    density: Density,
    initial_fraction_ionized_hydrogen: Dimensionless,
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
    box_size: Res<SimulationBox>,
) {
    for (entity, pos) in particles.iter() {
        commands.entity(entity).insert((
            components::Density(parameters.density),
            components::HydrogenIonizationFraction(parameters.initial_fraction_ionized_hydrogen),
        ));
        let pos_frac_x = (pos.x() / box_size.side_lengths().x()).value();
        let pos_frac_y = (pos.y() / box_size.side_lengths().y()).value();
        if 0.49 < pos_frac_x && pos_frac_x < 0.51 {
            if 0.49 < pos_frac_y && pos_frac_y < 0.51 {
                commands
                    .entity(entity)
                    .insert(components::Source(1.0e-7 / Time::seconds(1.0)));
            }
        }
    }
}
