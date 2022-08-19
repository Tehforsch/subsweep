use bevy::prelude::Commands;
use bevy::prelude::Res;
use bevy::prelude::*;
use mpi::Rank;
use rand::Rng;

use crate::mass::Mass;
use crate::particle::LocalParticleBundle;
use crate::position::Position;
use crate::units::f32::kilogram;
use crate::units::vec2::meter;
use crate::units::vec2::meters_per_second;
use crate::velocity::Velocity;

pub struct InitialConditionsPlugin;

impl Plugin for InitialConditionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_particles_system);
    }
}

fn spawn_particles_system(mut commands: Commands, rank: Res<Rank>) {
    if *rank != 0 {
        return;
    }
    let n_particles = 10;
    for _ in 0..n_particles {
        let x = rand::thread_rng().gen_range(-3.0..3.0);
        let y = rand::thread_rng().gen_range(-3.0..3.0);
        let pos = meter(Vec2::new(x, y));
        let x = rand::thread_rng().gen_range(-3.0..3.0);
        let y = rand::thread_rng().gen_range(-3.0..3.0);
        let vel = meters_per_second(Vec2::new(x, y)) * 0.0;
        commands.spawn().insert_bundle(LocalParticleBundle::new(
            Position(pos),
            Velocity(vel),
            Mass(kilogram(1000000.0)),
        ));
    }
}
