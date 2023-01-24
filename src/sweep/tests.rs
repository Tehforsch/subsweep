use bevy::prelude::Commands;
use bevy::prelude::Entity;
use bevy::prelude::Res;

use crate::components;
use crate::components::Position;
use crate::grid::init_cartesian_grid_system;
use crate::parameters::SimulationBox;
use crate::parameters::SimulationParameters;
use crate::parameters::SweepParameters;
use crate::parameters::TimestepParameters;
use crate::prelude::Particles;
use crate::prelude::SimulationStartupStages;
use crate::simulation::Simulation;
use crate::stages::SimulationStagesPlugin;
use crate::sweep::parameters::DirectionsSpecification;
use crate::sweep::SweepPlugin;
use crate::units::Density;
use crate::units::Dimensionless;
use crate::units::Length;
use crate::units::MVec;
use crate::units::Time;
use crate::units::VecDimensionless;

fn run_sweep(dirs: Vec<VecDimensionless>) {
    let num_cells = 10;
    let cell_size = Length::meters(0.1);
    let simulation_box = SimulationBox::cube_from_side_length(cell_size * num_cells as f64);
    let mut sim = Simulation::test();
    sim.add_parameter_file_contents("".into())
        .add_plugin(SimulationStagesPlugin)
        .add_parameters_explicitly(simulation_box)
        .add_parameters_explicitly(SweepParameters {
            directions: DirectionsSpecification::Explicit(dirs),
        })
        .add_parameters_explicitly(SimulationParameters { final_time: None })
        .add_parameters_explicitly(TimestepParameters {
            max_timestep: Time::seconds(1e-3),
        })
        .add_startup_system(move |commands: Commands, box_size: Res<SimulationBox>| {
            init_cartesian_grid_system(commands, box_size, cell_size)
        })
        .add_startup_system_to_stage(
            SimulationStartupStages::InsertDerivedComponents,
            initialize_sweep_components_system,
        )
        .add_plugin(SweepPlugin);
    sim.update();
}

fn initialize_sweep_components_system(
    mut commands: Commands,
    particles: Particles<(Entity, &Position)>,
) {
    for (entity, _) in particles.iter() {
        commands.entity(entity).insert((
            components::Density(Density::zero()),
            components::IonizedHydrogenFraction(Dimensionless::zero()),
        ));
    }
}

#[test]
fn simple_sweep() {
    run_sweep(vec![MVec::ONE * Dimensionless::dimensionless(1.0)]);
}

#[test]
fn sweep_along_grid_axes_does_not_deadlock_or_crash() {
    run_sweep(vec![MVec::X * Dimensionless::dimensionless(1.0)]);
}
