use std::time::Duration;

use bevy_ecs::prelude::Commands;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BatchSize;
use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::Throughput;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use subsweep::communication::BaseCommunicationPlugin;
use subsweep::components::Position;
use subsweep::domain::DomainPlugin;
use subsweep::parameters::Cosmology;
use subsweep::parameters::SimulationBoxParameters;
use subsweep::parameters::SimulationParameters;
use subsweep::parameters::SweepParameters;
use subsweep::prelude::LocalParticle;
use subsweep::prelude::ParallelVoronoiGridConstruction;
use subsweep::prelude::ParticleId;
use subsweep::prelude::Simulation;
use subsweep::prelude::StartupStages;
use subsweep::simulation_plugin::SimulationPlugin;
use subsweep::sweep::initialize_sweep_test_components_system;
use subsweep::sweep::DirectionsSpecification;
use subsweep::sweep::SweepPlugin;
use subsweep::units::Dimensionless;
use subsweep::units::Length;
use subsweep::units::PhotonRate;
use subsweep::units::Time;
use subsweep::units::VecLength;
use subsweep::voronoi::Point3d;

pub const NUM_DIRS: usize = 84;

fn setup_sweep_sim(num_particles: usize) -> Simulation {
    let mut sim = Simulation::default();
    let dirs = DirectionsSpecification::Num(NUM_DIRS);
    let num_timestep_levels = 3;
    let timestep_safety_factor = Dimensionless::dimensionless(0.1);
    sim.write_output(false)
        .add_parameter_file_contents("{}".into())
        .add_plugin(DomainPlugin)
        .add_plugin(BaseCommunicationPlugin::new(1, 0))
        .add_parameters_explicitly(SimulationBoxParameters::Normal(Length::meters(1e5)))
        .add_parameters_explicitly(SweepParameters {
            directions: dirs,
            rotate_directions: false,
            num_timestep_levels,
            significant_rate_threshold: PhotonRate::zero(),
            timestep_safety_factor,
            chemistry_timestep_safety_factor: timestep_safety_factor,
            max_timestep: Time::seconds(1e-3),
            check_deadlock: false,
            periodic: false,
            prevent_cooling: false,
            num_tasks_to_solve_before_send_receive: 10000,
        })
        .add_parameters_explicitly(Cosmology::NonCosmological)
        .add_parameters_explicitly(SimulationParameters { final_time: None })
        .add_startup_system_to_stage(
            StartupStages::InsertComponentsAfterGrid,
            initialize_sweep_test_components_system,
        )
        .add_startup_system_to_stage(StartupStages::ReadInput, move |commands: Commands| {
            insert_particles_system(commands, num_particles)
        })
        .add_plugin(ParallelVoronoiGridConstruction)
        .add_plugin(SimulationPlugin)
        .add_plugin(SweepPlugin);
    sim.update();
    sim
}

fn run_sim(mut sim: Simulation) {
    for _ in 0..10 {
        sim.update();
    }
}

pub fn sweep_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("sweep");
    group
        .noise_threshold(0.05)
        .measurement_time(Duration::from_secs(20))
        .sample_size(10);
    for num_particles in [500, 2000] {
        group.throughput(Throughput::Elements(num_particles as u64 * NUM_DIRS as u64));
        group.bench_function(BenchmarkId::from_parameter(num_particles), |b| {
            b.iter_batched(
                || setup_sweep_sim(num_particles),
                run_sim,
                BatchSize::LargeInput,
            )
        });
    }
    group.finish();
}

criterion_group!(benches, sweep_benchmark);
criterion_main!(benches);

fn insert_particles_system(mut commands: Commands, num_particles: usize) {
    let points = setup_particles_3d(num_particles);
    for (i, p) in points.into_iter().enumerate() {
        commands.spawn((
            Position(VecLength::new_unchecked(p)),
            ParticleId::test(i),
            LocalParticle,
        ));
    }
}

fn setup_particles_3d(num_particles: usize) -> Vec<Point3d> {
    let mut rng = StdRng::seed_from_u64(1338);
    (0..num_particles)
        .map(|_| {
            let x = rng.gen_range(0.0..1.0e5);
            let y = rng.gen_range(0.0..1.0e5);
            let z = rng.gen_range(0.0..1.0e5);
            Point3d::new(x, y, z)
        })
        .collect()
}
