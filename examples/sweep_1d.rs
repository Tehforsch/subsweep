use bevy::prelude::*;
use ordered_float::OrderedFloat;
use raxiom::components;
use raxiom::components::Density;
use raxiom::components::Position;
use raxiom::components::Source;
use raxiom::parameters::Cosmology;
use raxiom::parameters::Fields;
use raxiom::parameters::HandleExistingOutput;
use raxiom::parameters::OutputParameters;
use raxiom::parameters::SimulationParameters;
use raxiom::parameters::SweepParameters;
use raxiom::prelude::Extent;
use raxiom::prelude::LocalParticle;
use raxiom::prelude::Particles;
use raxiom::prelude::Simulation;
use raxiom::prelude::SimulationBox;
use raxiom::prelude::SimulationBuilder;
use raxiom::prelude::StartupStages;
use raxiom::prelude::WorldRank;
use raxiom::prelude::WorldSize;
use raxiom::sweep::grid::init_cartesian_grid_system;
use raxiom::sweep::grid::NumCellsSpec;
use raxiom::sweep::DirectionsSpecification;
use raxiom::sweep::SweepPlugin;
use raxiom::units::Dimensionless;
use raxiom::units::Length;
use raxiom::units::Mass;
use raxiom::units::PhotonFlux;
use raxiom::units::PhotonRate;
use raxiom::units::SourceRate;
use raxiom::units::Temperature;
use raxiom::units::Time;
use raxiom::units::VecLength;
use raxiom::units::Volume;

const BOX_SIZE_MEGAPARSEC: f64 = 1.0;
const FLUX_PHOTONS_PER_S_PER_CM_2: f64 = 1e4;

fn main() {
    let mut sim = setup_sweep_sim(10000);
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
        .verbosity(2)
        .build_with_sim(&mut sim_);
    let dirs = DirectionsSpecification::Num(1);
    let num_timestep_levels = 3;
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
            max_timestep: Time::megayears(1.0e-1),
            check_deadlock: false,
            periodic: false,
        })
        .add_parameters_explicitly(OutputParameters {
            time_between_snapshots: Time::megayears(1.0),
            time_first_snapshot: None,
            output_dir: "output".into(),
            snapshots_dir: "snapshots".into(),
            time_series_dir: "time_series".into(),
            fields: Fields::All,
            snapshot_padding: 4,
            used_parameters_filename: "parameters.yml".into(),
            handle_existing_output: HandleExistingOutput::Delete,
        })
        .add_parameters_explicitly(Cosmology::NonCosmological)
        .add_parameters_explicitly(SimulationParameters {
            final_time: Some(Time::megayears(1000.0)),
        })
        .add_startup_system_to_stage(
            StartupStages::InsertComponentsAfterGrid,
            initialize_sweep_components_system,
        )
        .add_startup_system_to_stage(StartupStages::InsertComponentsAfterGrid, add_source_system)
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

fn add_source_system(mut commands: Commands, particles: Particles<(Entity, &Position)>) {
    let most_left = particles
        .iter()
        .min_by_key(|(_, pos)| OrderedFloat(pos.x().value_unchecked()))
        .unwrap();

    for (entity, _) in particles.iter() {
        let source = if entity != most_left.0 {
            SourceRate::zero()
        } else {
            let num_particles = particles.iter().count();
            let cell_size = Length::megaparsec(BOX_SIZE_MEGAPARSEC) / num_particles as f64;
            let photons_per_second =
                PhotonFlux::photons_per_s_per_cm_squared(FLUX_PHOTONS_PER_S_PER_CM_2)
                    * cell_size.squared();
            photons_per_second
        };
        commands.entity(entity).insert(Source(source));
    }
}

pub fn initialize_sweep_components_system(
    mut commands: Commands,
    local_particles: Query<Entity, With<LocalParticle>>,
) {
    for entity in local_particles.iter() {
        commands.entity(entity).insert((
            Density(Mass::grams(1.0e-27) / Volume::cubic_centimeters(1.0)),
            components::IonizedHydrogenFraction(Dimensionless::dimensionless(1e-10)),
            components::Temperature(Temperature::kelvins(1000.0)),
            components::PhotonRate(PhotonRate::zero()),
        ));
    }
}
