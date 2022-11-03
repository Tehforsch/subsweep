use std::time::Duration;

use bevy::prelude::*;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BatchSize;
use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::Throughput;
use raxiom::ics::ConstantDensity;
use raxiom::ics::Resolution;
use raxiom::ics::Sampler;
use raxiom::ics::ZeroVelocity;
use raxiom::parameters::DomainParameters;
use raxiom::parameters::GravityParameters;
use raxiom::parameters::PerformanceParameters;
use raxiom::parameters::SimulationParameters;
use raxiom::parameters::TimestepParameters;
use raxiom::prelude::Extent;
use raxiom::prelude::GravityPlugin;
use raxiom::prelude::MVec;
use raxiom::prelude::Simulation;
use raxiom::prelude::SimulationBuilder;
use raxiom::prelude::WorldRank;
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
    let mut builder = SimulationBuilder::new();
    let mut sim = Simulation::default();
    sim.add_parameters_explicitly(PerformanceParameters::default())
        .add_parameters_explicitly(DomainParameters::default())
        .add_parameters_explicitly(GravityParameters {
            softening_length: Length::zero(),
            opening_angle,
        })
        .add_parameters_explicitly(SimulationParameters {
            final_time: Some(Time::seconds(10e-3)),
        })
        .add_parameters_explicitly(TimestepParameters {
            max_timestep: Time::seconds(1e-3),
            num_levels: 1,
        });
    builder
        .read_initial_conditions(false)
        .write_output(false)
        .headless(true)
        .log(false)
        .build_with_sim(&mut sim)
        .add_startup_system(move |commands: Commands, rank: Res<WorldRank>| {
            initial_conditions_system(commands, rank, num_particles)
        })
        .add_plugin(GravityPlugin);
    sim
}

fn initial_conditions_system(mut commands: Commands, rank: Res<WorldRank>, num_particles: usize) {
    if !rank.is_main() {
        return;
    }
    let box_size = Length::meters(100.0) * MVec::ONE;
    Sampler::new(
        ConstantDensity(Density::kilogram_per_cubic_meter(1.0)),
        ZeroVelocity,
        Extent::new(-box_size / 2.0, box_size / 2.0),
        Resolution::NumParticles(num_particles),
    )
    .spawn(&mut commands);
}
