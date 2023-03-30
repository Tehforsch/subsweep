use bevy::prelude::Commands;
use bevy::prelude::Res;

use crate::communication::local_sim_building::build_local_communication_sim_with_custom_logic;
use crate::grid::init_cartesian_grid_system;
use crate::grid::NumCellsSpec;
use crate::parameters::DomainParameters;
use crate::parameters::DomainStage;
use crate::parameters::SimulationBox;
use crate::parameters::SimulationParameters;
use crate::parameters::SweepParameters;
use crate::parameters::TimestepParameters;
use crate::prelude::SimulationStartupStages;
use crate::prelude::WorldRank;
use crate::prelude::WorldSize;
use crate::simulation::Simulation;
use crate::stages::SimulationStagesPlugin;
use crate::sweep::initialize_sweep_components_system;
use crate::sweep::parameters::DirectionsSpecification;
use crate::sweep::SweepPlugin;
use crate::units::Dimensionless;
use crate::units::Length;
use crate::units::MVec;
use crate::units::PhotonFlux;
use crate::units::Time;
use crate::units::VecDimensionless;

fn build_sweep_sim(dirs: Vec<VecDimensionless>, sim: &mut Simulation, num_cells: usize) {
    let cell_size = Length::meters(0.1);
    let simulation_box = SimulationBox::cube_from_side_length(cell_size * num_cells as f64);
    sim.add_parameter_file_contents("".into())
        .add_parameters_explicitly(DomainParameters {
            stage: DomainStage::Update,
            ..Default::default()
        })
        .add_plugin(SimulationStagesPlugin)
        .add_parameters_explicitly(simulation_box)
        .add_parameters_explicitly(SweepParameters {
            directions: DirectionsSpecification::Explicit(dirs),
            num_timestep_levels: 1,
            significant_flux_treshold: PhotonFlux::zero(),
            timestep_safety_factor: Dimensionless::dimensionless(0.0),
        })
        .add_parameters_explicitly(SimulationParameters { final_time: None })
        .add_parameters_explicitly(TimestepParameters {
            max_timestep: Time::seconds(1e-3),
        })
        .add_startup_system(
            move |commands: Commands,
                  box_size: Res<SimulationBox>,
                  world_size: Res<WorldSize>,
                  world_rank: Res<WorldRank>| {
                init_cartesian_grid_system(
                    commands,
                    box_size,
                    NumCellsSpec::CellSize(cell_size),
                    world_size,
                    world_rank,
                )
            },
        )
        .add_startup_system_to_stage(
            SimulationStartupStages::InsertComponentsAfterGrid,
            initialize_sweep_components_system,
        )
        .add_plugin(SweepPlugin);
}

#[test]
#[ignore]
fn simple_sweep() {
    for num_ranks in 1..10 {
        println!("Running on {}", num_ranks);
        build_local_communication_sim_with_custom_logic(
            |sim: &mut Simulation| {
                build_sweep_sim(vec![MVec::ONE * Dimensionless::dimensionless(1.0)], sim, 10)
            },
            |mut sim| {
                sim.update();
            },
            num_ranks,
        );
    }
}

#[test]
#[ignore]
fn sweep_along_grid_axes_does_not_deadlock_or_crash() {
    build_local_communication_sim_with_custom_logic(
        |sim: &mut Simulation| {
            build_sweep_sim(vec![MVec::X * Dimensionless::dimensionless(1.0)], sim, 5)
        },
        |mut sim| {
            sim.update();
        },
        2,
    );
}
