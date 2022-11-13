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
use raxiom::parameters::HydrodynamicsParameters;
use raxiom::parameters::InitialGasEnergy;
use raxiom::parameters::PerformanceParameters;
use raxiom::parameters::QuadTreeConfig;
use raxiom::parameters::SimulationBox;
use raxiom::parameters::SimulationParameters;
use raxiom::parameters::TimestepParameters;
use raxiom::prelude::HydrodynamicsPlugin;
use raxiom::prelude::Simulation;
use raxiom::prelude::SimulationBuilder;
use raxiom::units::Time;
use raxiom::units::*;

pub fn hydrodynamics_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("hydrodynamics");
    group
        .noise_threshold(0.05)
        .measurement_time(Duration::from_secs(20))
        .sample_size(10);
    for num_particles in [100, 1000, 10000] {
        group.throughput(Throughput::Elements(num_particles as u64));
        group.bench_function(BenchmarkId::from_parameter(num_particles), |b| {
            b.iter_batched(
                || setup_hydro_sim(num_particles),
                run_hydro,
                BatchSize::LargeInput,
            )
        });
    }
    Simulation::finalize();
    group.finish();
}

criterion_group!(benches, hydrodynamics_benchmark);
criterion_main!(benches);

fn run_hydro(mut sim: Simulation) {
    sim.run_without_finalize();
}

fn setup_hydro_sim(num_particles: usize) -> Simulation {
    let mut sim = Simulation::default();
    sim.add_parameters_explicitly(PerformanceParameters::default())
        .add_parameters_explicitly(DomainParameters::default())
        .add_parameters_explicitly(SimulationBox::cube_from_side_length(Length::meters(100.0)))
        .add_parameters_explicitly(HydrodynamicsParameters {
            min_smoothing_length: Length::meters(1.0),
            initial_gas_energy: InitialGasEnergy::TemperatureAndMolecularWeight {
                temperature: Temperature::kelvins(1e5),
                molecular_weight: Dimensionless::dimensionless(1.0),
            },
            tree: QuadTreeConfig::default(),
        })
        .add_parameters_explicitly(SimulationParameters {
            final_time: Some(Time::seconds(10e-3)),
        })
        .add_parameters_explicitly(TimestepParameters {
            max_timestep: Time::seconds(1e-3),
            num_levels: 1,
        });
    SimulationBuilder::bench()
        .build_with_sim(&mut sim)
        .add_plugin(
            InitialConditionsPlugin::default()
                .density_profile(ConstantDensity(Density::kilogram_per_cubic_meter(1.0)))
                .sampler(MonteCarloSampler::num_particles(num_particles)),
        )
        .add_plugin(HydrodynamicsPlugin);
    sim
}
