use bevy::prelude::App;
use bevy::prelude::Commands;
use bevy::MinimalPlugins;

use super::tests::get_particles;
use crate::communication::build_local_communication_app_with_custom_logic;
use crate::domain::DomainDecompositionPlugin;
use crate::mass;
use crate::output;
use crate::physics::gravity::plugin::GravityPlugin;
use crate::physics::PhysicsPlugin;
use crate::physics::{self};
use crate::position::Position;
use crate::units::VecVelocity;
use crate::velocity::Velocity;

#[test]
#[cfg(not(feature = "mpi"))]
fn compare_parallel_quadtree_gravity_to_direct_sum() {
    let check = |mut app: App| {
        app.run();
    };
    build_local_communication_app_with_custom_logic(build_parallel_gravity_app, check, 4);
    assert!(false);
}

fn spawn_particles_system(mut commands: Commands) {
    commands.spawn_batch(get_particles(5).into_iter().map(|part| {
        (
            Position(part.pos),
            mass::Mass(part.mass),
            Velocity(VecVelocity::zero()),
        )
    }));
}

#[cfg(not(feature = "mpi"))]
fn build_parallel_gravity_app(app: &mut App) {
    use crate::quadtree::QuadTreeConfig;
    use crate::units::Dimensionless;

    app.insert_resource(physics::Parameters {
        opening_angle: Dimensionless::zero(),
        ..Default::default()
    })
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
