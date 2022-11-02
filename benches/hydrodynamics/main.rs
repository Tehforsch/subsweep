use std::time::Duration;

use bevy::prelude::*;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::Throughput;
use raxiom::components;
use raxiom::components::Position;
use raxiom::parameters::DomainParameters;
use raxiom::parameters::HydrodynamicsParameters;
use raxiom::parameters::InitialGasEnergy;
use raxiom::parameters::PerformanceParameters;
use raxiom::parameters::QuadTreeConfig;
use raxiom::parameters::SimulationParameters;
use raxiom::parameters::TimestepParameters;
use raxiom::prelude::gen_range;
use raxiom::prelude::HydrodynamicsPlugin;
use raxiom::prelude::LocalParticle;
use raxiom::prelude::MVec;
use raxiom::prelude::Simulation;
use raxiom::prelude::SimulationBuilder;
use raxiom::prelude::WorldRank;
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
            b.iter(|| run_hydro(setup_hydro_sim(num_particles)))
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
    let mut builder = SimulationBuilder::new();
    let mut sim = Simulation::default();
    sim.add_parameters_explicitly(PerformanceParameters::default())
        .add_parameters_explicitly(DomainParameters::default())
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
    builder
        .read_initial_conditions(false)
        .write_output(false)
        .headless(true)
        .log(false)
        .build_with_sim(&mut sim)
        .add_startup_system(move |commands: Commands, rank: Res<WorldRank>| {
            spawn_particles_system(commands, rank, num_particles)
        })
        .add_plugin(HydrodynamicsPlugin);
    sim
}

fn spawn_particles_system(mut commands: Commands, rank: Res<WorldRank>, num_particles: usize) {
    if !rank.is_main() {
        return;
    }
    let box_size = Length::meters(100.0) * MVec::ONE;
    for _ in 0..num_particles {
        let pos = gen_range(-box_size, box_size);
        spawn_particle(
            &mut commands,
            pos,
            VecVelocity::zero(),
            Mass::kilograms(1.0),
        )
    }
}

fn spawn_particle(commands: &mut Commands, pos: VecLength, vel: VecVelocity, mass: Mass) {
    commands.spawn_bundle((
        LocalParticle,
        Position(pos),
        components::Velocity(vel),
        components::Mass(mass),
    ));
}
