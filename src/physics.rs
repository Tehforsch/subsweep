use bevy::prelude::Plugin;
use bevy::prelude::Query;
use bevy::prelude::Res;
use mpi::traits::Equivalence;

use crate::position::Position;
use crate::units::f32::second;
use crate::units::f32::Time;
use crate::velocity::Velocity;

#[derive(Equivalence)]
struct Timestep(Time);

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Timestep(second(0.001)))
            .add_system(integrate_motion_system);
    }
}

fn integrate_motion_system(mut query: Query<(&mut Position, &Velocity)>, timestep: Res<Timestep>) {
    for (mut pos, velocity) in query.iter_mut() {
        pos.0 += velocity.0 * timestep.0;
    }
}
