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
use crate::gravity::plugin::GravityPlugin;
use crate::gravity::GravityParameters;
use crate::gravity::LeafData;
use crate::gravity::Solver;
use crate::prelude::Extent;
use crate::prelude::LocalParticle;
use crate::prelude::Particles;
use crate::prelude::SimulationBox;
use crate::simulation::Simulation;
use crate::simulation_plugin::SimulationPlugin;
use crate::simulation_plugin::TimestepParameters;
use crate::test_utils::run_system_on_sim;
use crate::units::VecVelocity;

pub const NUM_PARTICLES_ONE_DIMENSION: i32 = 20;

fn check_system(
    parameters: Res<GravityParameters>,
    query: Particles<(&Velocity, &IndexIntoArray)>,
    box_: Res<SimulationBox>,
    timestep: Res<TimestepParameters>,
) {
    let solver = Solver::new(&parameters, &box_);
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
        let acc2 = **vel / timestep.max_timestep;
        compare_accelerations(acc1, acc2);
    }
    // Check that we haven't accidentally broken this test by removing all the particles
    assert!(query.iter().count() > 0);
}

#[derive(Component, Equivalence, Clone, Debug)]
struct IndexIntoArray(usize);

fn get_particles_this_test() -> Vec<LeafData> {
    get_particles(NUM_PARTICLES_ONE_DIMENSION, NUM_PARTICLES_ONE_DIMENSION)
}

fn get_extent_this_test() -> Extent {
    Extent::from_positions(get_particles_this_test().iter().map(|x| &x.pos)).unwrap()
}

fn spawn_particles_system(rank: Res<WorldRank>, mut commands: Commands) {
    if **rank == 0 {
        commands.spawn_batch(get_particles_this_test().into_iter().enumerate().map(
            |(i, LeafData { pos, mass, .. })| {
                let vel = Velocity(VecVelocity::zero());
                (
                    Position(pos),
                    components::Mass(mass),
                    vel,
                    LocalParticle,
                    IndexIntoArray(i),
                )
            },
        ))
    }
}

#[cfg(not(feature = "mpi"))]
fn build_parallel_gravity_sim(sim: &mut Simulation) {
    use crate::domain::ExchangeDataPlugin;
    use crate::stages::SimulationStagesPlugin;
    use crate::units::Dimensionless;
    use crate::units::Length;
    use crate::units::Time;

    sim.add_parameter_file_contents("".into())
        .add_parameters_explicitly(TimestepParameters {
            max_timestep: Time::seconds(1.0),
        })
        .add_parameters_explicitly(GravityParameters {
            opening_angle: Dimensionless::dimensionless(0.0),
            softening_length: Length::meters(1e-30),
        })
        .add_parameters_explicitly(SimulationBox::from(get_extent_this_test()))
        .write_output(false)
        .add_startup_system(spawn_particles_system)
        .add_bevy_plugins(MinimalPlugins)
        .add_plugin(SimulationStagesPlugin)
        .add_plugin(DomainDecompositionPlugin)
        .add_plugin(SimulationPlugin)
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
