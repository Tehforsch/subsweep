use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Res;
use bevy::MinimalPlugins;
use mpi::traits::Equivalence;

use super::compare_accelerations;
use super::direct_sum;
use super::get_particles;
use crate::communication::local_sim_building::build_local_communication_sim_with_custom_logic;
use crate::communication::WorldRank;
use crate::components;
use crate::components::Position;
use crate::components::Velocity;
use crate::domain::DomainDecompositionPlugin;
use crate::physics::gravity::plugin::GravityPlugin;
use crate::physics::gravity::GravityParameters;
use crate::physics::gravity::LeafData;
use crate::physics::gravity::Solver;
use crate::physics::PhysicsPlugin;
use crate::physics::Timestep;
use crate::prelude::LocalParticle;
use crate::prelude::Particles;
use crate::simulation::Simulation;
use crate::test_utils::run_system_on_sim;
use crate::units::VecVelocity;

pub const NUM_PARTICLES_ONE_DIMENSION: i32 = 20;

fn check_system(
    parameters: Res<GravityParameters>,
    timestep: Res<Timestep>,
    query: Particles<(&Velocity, &IndexIntoArray)>,
) {
    let solver = Solver::from_parameters(&parameters);
    for (vel, index) in query.iter() {
        let particles = get_particles(NUM_PARTICLES_ONE_DIMENSION, NUM_PARTICLES_ONE_DIMENSION);
        // We can't use the particle position from a query here,
        // because that has already been integrated
        let pos = &particles[index.0].pos;
        let direct_sum = direct_sum(
            &solver,
            pos,
            particles
                .iter()
                .map(|LeafData { pos, mass, .. }| (*pos, *mass))
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
            get_particles(NUM_PARTICLES_ONE_DIMENSION, NUM_PARTICLES_ONE_DIMENSION)
                .into_iter()
                .enumerate()
                .map(|(i, LeafData { pos, mass, .. })| {
                    let vel = Velocity(VecVelocity::zero());
                    (
                        Position(pos),
                        components::Mass(mass),
                        vel,
                        LocalParticle,
                        IndexIntoArray(i),
                    )
                }),
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
