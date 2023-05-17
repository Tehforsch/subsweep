use std::time::Duration;

use bevy::prelude::Commands;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BatchSize;
use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::Throughput;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use raxiom::communication::BaseCommunicationPlugin;
use raxiom::components::Position;
use raxiom::domain::DomainPlugin;
use raxiom::parameters::SimulationBox;
use raxiom::parameters::SimulationParameters;
use raxiom::parameters::SweepParameters;
use raxiom::parameters::TimestepParameters;
use raxiom::parameters::TreeParameters;
use raxiom::prelude::LocalParticle;
use raxiom::prelude::ParallelVoronoiGridConstruction;
use raxiom::prelude::ParticleId;
use raxiom::prelude::Simulation;
use raxiom::prelude::StartupStages;
use raxiom::simulation_plugin::SimulationPlugin;
use raxiom::stages::SimulationStagesPlugin;
use raxiom::sweep::initialize_sweep_components_system;
use raxiom::sweep::DirectionsSpecification;
use raxiom::sweep::SweepPlugin;
use raxiom::units::Dimensionless;
use raxiom::units::Length;
use raxiom::units::PhotonRate;
use raxiom::units::Time;
use raxiom::units::VecLength;
use raxiom::voronoi::Point3d;

fn setup_sweep_sim(num_particles: usize) -> Simulation {
    let mut sim = Simulation::default();
    let dirs = DirectionsSpecification::Num(84);
    let num_timestep_levels = 3;
    let timestep_safety_factor = Dimensionless::dimensionless(0.1);
    sim.add_parameter_file_contents("{}".into())
        .add_plugin(SimulationStagesPlugin)
        .add_plugin(DomainPlugin)
        .add_plugin(BaseCommunicationPlugin::new(1, 0))
        .add_parameters_explicitly(TreeParameters {
            ..Default::default()
        })
        .add_parameters_explicitly(SimulationBox::cube_from_side_length(Length::meters(1e5)))
        .add_parameters_explicitly(SweepParameters {
            directions: dirs.clone(),
            num_timestep_levels: num_timestep_levels,
            significant_rate_treshold: PhotonRate::zero(),
            timestep_safety_factor: timestep_safety_factor,
            check_deadlock: false,
        })
        .add_parameters_explicitly(SimulationParameters { final_time: None })
        .add_parameters_explicitly(TimestepParameters {
            max_timestep: Time::seconds(1e-3),
        })
        .add_startup_system_to_stage(
            StartupStages::InsertComponentsAfterGrid,
            initialize_sweep_components_system,
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
    for _ in 0..5 {
        sim.update();
    }
}

pub fn sweep_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("sweep");
    group
        .noise_threshold(0.05)
        .measurement_time(Duration::from_secs(20))
        .sample_size(10);
    for num_particles in [1000, 10000] {
        group.throughput(Throughput::Elements(num_particles as u64));
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
