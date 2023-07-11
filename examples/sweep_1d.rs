use bevy::prelude::Commands;
use bevy::prelude::Res;
use raxiom::parameters::Cosmology;
use raxiom::parameters::SimulationParameters;
use raxiom::parameters::SweepParameters;
use raxiom::prelude::Extent;
use raxiom::prelude::Simulation;
use raxiom::prelude::SimulationBox;
use raxiom::prelude::SimulationBuilder;
use raxiom::prelude::StartupStages;
use raxiom::prelude::WorldRank;
use raxiom::prelude::WorldSize;
use raxiom::sweep::grid::init_cartesian_grid_system;
use raxiom::sweep::grid::NumCellsSpec;
use raxiom::sweep::initialize_sweep_test_components_system;
use raxiom::sweep::DirectionsSpecification;
use raxiom::sweep::SweepPlugin;
use raxiom::units::Dimensionless;
use raxiom::units::Length;
use raxiom::units::PhotonRate;
use raxiom::units::Time;
use raxiom::units::VecLength;

const BOX_SIZE_MEGAPARSEC: f64 = 1.0;

fn main() {
    let mut sim = setup_sweep_sim(1000);
    sim.run();
}

fn setup_sweep_sim(num_particles: usize) -> Simulation {
    let mut sim_ = Simulation::default();
    add_box_size(&mut sim_, num_particles);
    let mut sim = SimulationBuilder::new();
    let mut sim = sim
        .write_output(true)
        .read_initial_conditions(false)
        .require_parameter_file(false)
        .build_with_sim(&mut sim_);
    let dirs = DirectionsSpecification::Num(1);
    let num_timestep_levels = 1;
    let timestep_safety_factor = Dimensionless::dimensionless(0.1);
    add_grid(&mut sim, num_particles);
    sim.write_output(true)
        .add_parameters_explicitly(SweepParameters {
            directions: dirs,
            rotate_directions: false,
            num_timestep_levels,
            significant_rate_threshold: PhotonRate::zero(),
            timestep_safety_factor,
            chemistry_timestep_safety_factor: timestep_safety_factor,
            max_timestep: Time::megayears(1e-1),
            check_deadlock: false,
            periodic: false,
        })
        .add_parameters_explicitly(Cosmology::NonCosmological)
        .add_parameters_explicitly(SimulationParameters {
            final_time: Some(Time::megayears(10.0)),
        })
        .add_startup_system_to_stage(
            StartupStages::InsertComponentsAfterGrid,
            initialize_sweep_test_components_system,
        )
        .add_plugin(SweepPlugin);
    sim_
}

fn add_box_size(sim: &mut Simulation, num_particles: usize) {
    let cell_size = Length::megaparsec(BOX_SIZE_MEGAPARSEC) / num_particles as f64;
    let min = VecLength::zero();
    let max = VecLength::new(cell_size * num_particles as f64, cell_size, cell_size);
    let extent = Extent {
        min,
        max,
        center: (min + max) / 2.0,
    };
    let box_size = SimulationBox::new(extent);
    sim.insert_resource(box_size);
}

fn add_grid(sim: &mut Simulation, num_particles: usize) {
    let cell_size = Length::megaparsec(BOX_SIZE_MEGAPARSEC) / num_particles as f64;
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
            false,
        )
    };
    sim.add_startup_system(grid_setup);
}
