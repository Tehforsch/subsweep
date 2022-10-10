use std::time::Duration;

use bevy::prelude::*;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use raxiom::parameters::DomainTreeParameters;
use raxiom::parameters::HydrodynamicsParameters;
use raxiom::parameters::PerformanceParameters;
use raxiom::parameters::QuadTreeConfig;
use raxiom::parameters::SimulationParameters;
use raxiom::prelude::gen_range;
use raxiom::prelude::HydrodynamicsPlugin;
use raxiom::prelude::LocalParticle;
use raxiom::prelude::MVec;
use raxiom::prelude::Position;
use raxiom::prelude::Simulation;
use raxiom::prelude::SimulationBuilder;
use raxiom::prelude::WorldRank;
use raxiom::units::Time;
use raxiom::units::*;

fn run_hydro() {
    let mut builder = SimulationBuilder::new();
    let mut sim = Simulation::default();
    sim.add_parameters_explicitly(PerformanceParameters::default())
        .add_parameters_explicitly(DomainTreeParameters::default())
        .add_parameters_explicitly(HydrodynamicsParameters {
            smoothing_length: Length::meters(1.0),
            tree: QuadTreeConfig::default(),
        })
        .add_parameters_explicitly(SimulationParameters {
            timestep: Time::seconds(1e-3),
            final_time: Some(Time::seconds(10e-3)),
        });
    builder
        .read_initial_conditions(false)
        .write_output(false)
        .headless(true)
        .log(false)
        .build_with_sim(&mut sim)
        .add_startup_system(spawn_particles_system)
        .add_plugin(HydrodynamicsPlugin)
        .run_without_finalize();
}

pub fn hydrodynamics_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("hydrodynamics");
    group
        .sample_size(50)
        .measurement_time(Duration::from_secs(60));
    group.bench_function("hydrodynamics", |b| b.iter(run_hydro));
    Simulation::finalize();
    group.finish();
}

criterion_group!(benches, hydrodynamics_benchmark);
criterion_main!(benches);

fn spawn_particles_system(mut commands: Commands, rank: Res<WorldRank>) {
    if !rank.is_main() {
        return;
    }
    let num_particles = 10000;
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
        raxiom::prelude::Velocity(vel),
        raxiom::prelude::Mass(mass),
    ));
}
