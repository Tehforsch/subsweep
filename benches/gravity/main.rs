use std::time::Duration;

use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BatchSize;
use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::Throughput;
use raxiom::ics::ConstantDensity;
use raxiom::ics::InitialConditionsPlugin;
use raxiom::ics::MonteCarloSampler;
use raxiom::parameters::DomainParameters;
use raxiom::parameters::GravityParameters;
use raxiom::parameters::PerformanceParameters;
use raxiom::parameters::SimulationParameters;
use raxiom::prelude::GravityPlugin;
use raxiom::prelude::Simulation;
use raxiom::prelude::SimulationBox;
use raxiom::prelude::SimulationBuilder;
use raxiom::simulation_plugin::TimestepParameters;
use raxiom::units::Time;
use raxiom::units::*;

pub fn gravity_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("gravity");
    group
        .noise_threshold(0.05)
        .measurement_time(Duration::from_secs(20))
        .sample_size(10);
    for num_particles in [100, 1000, 10000] {
        group.throughput(Throughput::Elements(num_particles as u64));
        group.bench_function(BenchmarkId::from_parameter(num_particles), |b| {
            b.iter_batched(
                || setup_gravity_sim(num_particles, Dimensionless::dimensionless(0.5)),
                run_gravity,
                BatchSize::LargeInput,
            )
        });
    }
    Simulation::finalize();
    group.finish();
}

criterion_group!(benches, gravity_benchmark);
criterion_main!(benches);

fn run_gravity(mut sim: Simulation) {
    sim.run_without_finalize();
}

fn setup_gravity_sim(num_particles: usize, opening_angle: Dimensionless) -> Simulation {
    let mut sim = Simulation::default();
    sim.add_parameters_explicitly(PerformanceParameters::default())
        .add_parameters_explicitly(DomainParameters::default())
        .add_parameters_explicitly(SimulationBox::cube_from_side_length(Length::meters(100.0)))
        .add_parameters_explicitly(GravityParameters {
            softening_length: Length::zero(),
            opening_angle,
        })
        .add_parameters_explicitly(SimulationParameters {
            final_time: Some(Time::seconds(10e-3)),
        })
        .add_parameters_explicitly(TimestepParameters {
            max_timestep: Time::years(1e-3),
        });
    SimulationBuilder::bench()
        .build_with_sim(&mut sim)
        .add_plugin(
            InitialConditionsPlugin::default()
                .density_profile(ConstantDensity(Density::kilogram_per_cubic_meter(1.0)))
                .sampler(MonteCarloSampler::num_particles(num_particles)),
        )
        .add_plugin(GravityPlugin);
    sim
}
