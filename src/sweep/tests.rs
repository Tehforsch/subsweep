use bevy::prelude::Commands;
use bevy::prelude::Res;

use crate::grid::init_cartesian_grid_system;
use crate::grid::NumCellsSpec;
use crate::parameters::SimulationBox;
use crate::parameters::SimulationParameters;
use crate::parameters::SweepParameters;
use crate::parameters::TimestepParameters;
use crate::parameters::TreeParameters;
use crate::prelude::SimulationStartupStages;
use crate::prelude::WorldRank;
use crate::prelude::WorldSize;
use crate::simulation::Simulation;
use crate::stages::SimulationStagesPlugin;
use crate::sweep::initialize_sweep_components_system;
use crate::sweep::parameters::DirectionsSpecification;
use crate::sweep::SweepPlugin;
use crate::test_utils::build_local_communication_sim_with_custom_logic;
use crate::units::Dimensionless;
use crate::units::Length;
use crate::units::MVec;
use crate::units::PhotonRate;
use crate::units::Time;
use crate::units::VecDimensionless;

struct SweepSetup {
    dirs: Vec<VecDimensionless>,
    num_timestep_levels: usize,
    timestep_safety_factor: Dimensionless,
    box_: SimulationBox,
}

fn setup_sweep_sim(sim: &mut Simulation, setup: SweepSetup) -> &mut Simulation {
    sim.add_parameter_file_contents("{}".into())
        .add_parameters_explicitly(TreeParameters {
            ..Default::default()
        })
        .add_plugin(SimulationStagesPlugin)
        .add_parameters_explicitly(setup.box_.clone())
        .add_parameters_explicitly(SweepParameters {
            directions: DirectionsSpecification::Explicit(setup.dirs.clone()),
            num_timestep_levels: setup.num_timestep_levels,
            significant_rate_treshold: PhotonRate::zero(),
            timestep_safety_factor: setup.timestep_safety_factor,
            check_deadlock: false,
        })
        .add_parameters_explicitly(SimulationParameters { final_time: None })
        .add_parameters_explicitly(TimestepParameters {
            max_timestep: Time::seconds(1e-3),
        })
        .add_startup_system_to_stage(
            SimulationStartupStages::InsertComponentsAfterGrid,
            initialize_sweep_components_system,
        )
        .add_plugin(SweepPlugin)
}

fn build_cartesian_sweep_sim(
    sim: &mut Simulation,
    dirs: Vec<VecDimensionless>,
    num_cells: usize,
    num_timestep_levels: usize,
    periodic: bool,
) {
    let cell_size = Length::meters(0.1);
    let simulation_box = SimulationBox::cube_from_side_length(cell_size * num_cells as f64);
    let grid_setup = move |commands: Commands,
                           box_size: Res<SimulationBox>,
                           world_size: Res<WorldSize>,
                           world_rank: Res<WorldRank>| {
        init_cartesian_grid_system(
            commands,
            box_size,
            NumCellsSpec::CellSize(cell_size),
            world_size,
            world_rank,
            periodic,
        )
    };
    setup_sweep_sim(
        sim,
        SweepSetup {
            dirs,
            num_timestep_levels,
            timestep_safety_factor: Dimensionless::zero(),
            box_: simulation_box,
        },
    );
    sim.add_startup_system(grid_setup);
}

#[test]
#[ignore]
fn simple_sweep() {
    for num_ranks in 1..10 {
        for num_timestep_levels in 1..3 {
            for periodic in [false, true] {
                println!("Running on {}", num_ranks);
                build_local_communication_sim_with_custom_logic(
                    move |sim: &mut Simulation| {
                        build_cartesian_sweep_sim(
                            sim,
                            vec![MVec::ONE * Dimensionless::dimensionless(1.0)],
                            10,
                            num_timestep_levels,
                            periodic,
                        )
                    },
                    |sim| {
                        sim.update();
                    },
                    num_ranks,
                );
            }
        }
    }
}

#[test]
#[ignore]
fn sweep_along_grid_axes_does_not_deadlock_or_crash() {
    build_local_communication_sim_with_custom_logic(
        |sim: &mut Simulation| {
            build_cartesian_sweep_sim(
                sim,
                vec![MVec::X * Dimensionless::dimensionless(1.0)],
                5,
                1,
                false,
            )
        },
        |sim| {
            sim.update();
        },
        2,
    );
}
