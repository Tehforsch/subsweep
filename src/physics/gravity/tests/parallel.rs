use bevy::ecs::schedule::IntoSystemDescriptor;
use bevy::prelude::App;
use bevy::prelude::Commands;
use bevy::prelude::Entity;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::Stage;
use bevy::prelude::SystemStage;
use bevy::MinimalPlugins;

use super::tests::compare_accelerations;
use super::tests::direct_sum;
use crate::communication::build_local_communication_app_with_custom_logic;
use crate::communication::WorldRank;
use crate::domain::DomainDecompositionPlugin;
use crate::mass;
use crate::output;
use crate::physics::gravity::plugin::GravityPlugin;
use crate::physics::gravity::Solver;
use crate::physics::LocalParticle;
use crate::physics::PhysicsPlugin;
use crate::physics::Timestep;
use crate::physics::{self};
use crate::position::Position;
use crate::units::Mass;
use crate::units::VecLength;
use crate::units::VecVelocity;
use crate::velocity::Velocity;

pub const NUM_PARTICLES_ONE_DIMENSION: usize = 6;

fn get_particles(n: usize) -> Vec<(Position, mass::Mass, Velocity)> {
    (0..n)
        .flat_map(move |x| {
            (0..n).map(move |y| {
                (
                    Position(VecLength::meter(x as f64, y as f64)),
                    mass::Mass(Mass::kilogram(1e11)),
                    Velocity(VecVelocity::zero()),
                )
            })
        })
        .collect()
}

fn run_system_on_app<P>(app: &mut App, system: impl IntoSystemDescriptor<P>) {
    let mut stage = SystemStage::parallel().with_system(system);
    stage.run(&mut app.world);
}

fn check_system(
    parameters: Res<physics::Parameters>,
    timestep: Res<Timestep>,
    query: Query<&Velocity>,
    entities: Res<Entities>,
) {
    let solver = Solver {
        softening_length: parameters.softening_length,
        opening_angle: parameters.opening_angle,
    };
    for (i, entity) in entities.0.iter().enumerate() {
        let vel = query.get(*entity).unwrap();
        // We can't use the particle position from a query here,
        // because that has already been integrated
        let particles = get_particles(NUM_PARTICLES_ONE_DIMENSION);
        let pos = &particles[i].0;
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
        dbg!(&acc1, &acc2);
        compare_accelerations(acc1, acc2);
    }
}

struct Entities(Vec<Entity>);

fn spawn_particles_system(
    rank: Res<WorldRank>,
    mut commands: Commands,
    mut remember_entities: ResMut<Entities>,
) {
    if **rank == 0 {
        for bundle in get_particles(NUM_PARTICLES_ONE_DIMENSION)
            .into_iter()
            .map(|(pos, mass, vel)| (pos, mass, vel, LocalParticle))
        {
            remember_entities.0.push(commands.spawn_bundle(bundle).id());
        }
    }
}

#[cfg(not(feature = "mpi"))]
fn build_parallel_gravity_app(app: &mut App) {
    use crate::quadtree::QuadTreeConfig;
    use crate::units::Dimensionless;
    use crate::units::Length;
    use crate::units::Time;

    app.insert_resource(physics::Parameters {
        timestep: Time::second(1.0),
        opening_angle: Dimensionless::dimensionless(0.0),
        softening_length: Length::meter(1.0),
        ..Default::default()
    })
    .insert_resource(Entities(vec![]))
    .insert_resource(QuadTreeConfig {
        ..Default::default()
    })
    .insert_resource(output::Parameters {
        ..Default::default()
    })
    .add_startup_system(spawn_particles_system)
    .add_plugins(MinimalPlugins)
    .add_plugin(DomainDecompositionPlugin)
    .add_plugin(PhysicsPlugin)
    .add_plugin(GravityPlugin);
}

#[test]
#[cfg(not(feature = "mpi"))]
fn compare_parallel_quadtree_gravity_to_direct_sum() {
    let check = |mut app: App| {
        app.update();
        run_system_on_app(&mut app, check_system);
    };
    build_local_communication_app_with_custom_logic(build_parallel_gravity_app, check, 2);
}
