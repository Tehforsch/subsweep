use bevy::prelude::Commands;
use bevy::prelude::Res;
use bevy::prelude::*;
use mpi::Rank;

use crate::domain::Domain;
use crate::mass::Mass;
use crate::particle::LocalParticleBundle;
use crate::position::Position;
use crate::units::f32::kilograms;
use crate::units::vec2::meters_per_second;
use crate::velocity::Velocity;

pub struct InitialConditionsPlugin;

impl Plugin for InitialConditionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_particles_system);
    }
}

fn spawn_particles_system(mut commands: Commands, domain: Res<Domain>, rank: Res<Rank>) {
    if *rank != 0 {
        return;
    }
    for i in 0..200 {
        let pos = domain.upper_left + (domain.lower_right - domain.upper_left) * i as f32;
        commands.spawn().insert_bundle(LocalParticleBundle::new(
            Position(pos),
            Velocity(meters_per_second(Vec2::new(1.0, 0.0))),
            Mass(kilograms(1.0)),
        ));
    }
}
