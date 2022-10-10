use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Res;
use bevy::MinimalPlugins;
use mpi::traits::Equivalence;

use super::compare_accelerations;
use super::direct_sum;
use crate::communication::local_sim_building::build_local_communication_sim_with_custom_logic;
use crate::communication::WorldRank;
use crate::domain::DomainDecompositionPlugin;
use crate::mass;
use crate::physics::gravity::plugin::GravityPlugin;
use crate::physics::gravity::GravityParameters;
use crate::physics::gravity::Solver;
use crate::physics::PhysicsPlugin;
use crate::physics::Timestep;
use crate::position::Position;
use crate::prelude::LocalParticle;
use crate::prelude::Particles;
use crate::simulation::Simulation;
use crate::test_utils::run_system_on_sim;
use crate::units::Mass;
use crate::units::VecLength;
use crate::units::VecVelocity;
use crate::velocity::Velocity;

pub const NUM_PARTICLES_ONE_DIMENSION: usize = 500;

fn get_particles(n: usize) -> Vec<(Position, mass::Mass, Velocity)> {
    (0..n)
        .map(move |x| {
            (
                Position(VecLength::meters(x as f64, 0.0 as f64)),
                mass::Mass(Mass::kilograms(1e11)),
                Velocity(VecVelocity::zero()),
            )
        })
        .collect()
}

fn check_system(
    parameters: Res<GravityParameters>,
    timestep: Res<Timestep>,
    query: Particles<(&Velocity, &IndexIntoArray)>,
) {
    let solver = Solver::from_parameters(&parameters);
    for (vel, index) in query.iter() {
        let particles = get_particles(NUM_PARTICLES_ONE_DIMENSION);
        // We can't use the particle position from a query here,
        // because that has already been integrated
        let pos = &particles[index.0].0;
        let direct_sum = direct_sum(
            &solver,
            pos,
            particles
                .iter()
                .map(|(pos, mass, _)| (**pos, **mass))
                .collect(),
        );
        let acc1 = direct_sum;
        let acc2 = **vel / **timestep;
        compare_accelerations(acc1, acc2);
    }
}

#[derive(Component, Equivalence, Clone, Debug)]
struct IndexIntoArray(usize);

fn spawn_particles_system(rank: Res<WorldRank>, mut commands: Commands) {
    if **rank == 0 {
        commands.spawn_batch(
            get_particles(NUM_PARTICLES_ONE_DIMENSION)
                .into_iter()
                .enumerate()
                .map(|(i, (pos, mass, vel))| (pos, mass, vel, LocalParticle, IndexIntoArray(i))),
        )
    }
}

#[cfg(not(feature = "mpi"))]
fn build_parallel_gravity_sim(sim: &mut Simulation) {
    use crate::domain::DomainTreeParameters;
    use crate::domain::ExchangeDataPlugin;
    use crate::io::output::ShouldWriteOutput;
    use crate::physics::SimulationParameters;
    use crate::stages::SimulationStagesPlugin;
    use crate::units::Dimensionless;
    use crate::units::Length;
    use crate::units::Time;

    sim.add_parameters_explicitly(SimulationParameters {
        timestep: Time::seconds(1.0),
        ..Default::default()
    })
    .add_parameters_explicitly(GravityParameters {
        opening_angle: Dimensionless::dimensionless(0.0),
        softening_length: Length::meters(1e-30),
    })
    .add_parameters_explicitly(DomainTreeParameters::default())
    .insert_resource(ShouldWriteOutput(false))
    .add_startup_system(spawn_particles_system)
    .add_bevy_plugins(MinimalPlugins)
    .add_plugin(SimulationStagesPlugin)
    .add_plugin(DomainDecompositionPlugin)
    .add_plugin(PhysicsPlugin)
    .add_plugin(GravityPlugin)
    .add_plugin(ExchangeDataPlugin::<IndexIntoArray>::default());
}

#[test]
#[ignore]
#[cfg(not(feature = "mpi"))]
fn compare_parallel_quadtree_gravity_to_direct_sum() {
    let check = |mut sim: Simulation| {
        sim.update();
        run_system_on_sim(&mut sim, check_system);
    };
    build_local_communication_sim_with_custom_logic(build_parallel_gravity_sim, check, 2);
}
