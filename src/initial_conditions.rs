use bevy::prelude::Commands;
use bevy::prelude::Res;
use bevy::prelude::*;
use rand::Rng;

use crate::communication::WorldRank;
use crate::mass::Mass;
use crate::particle::LocalParticleBundle;
use crate::position::Position;
use crate::units;
use crate::units::Vec2Length;
use crate::units::Vec2Velocity;
use crate::velocity::Velocity;

pub struct InitialConditionsPlugin;

impl Plugin for InitialConditionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_particles_system);
    }
}

fn spawn_particles_system(mut commands: Commands, rank: Res<WorldRank>) {
    if !rank.is_main() {
        return;
    }
    let n_particles = 150;
    for _ in 0..n_particles {
        let x = rand::thread_rng().gen_range(-5.0..-4.0);
        let y = rand::thread_rng().gen_range(-1.0..1.0);
        let pos = Vec2Length::meter(x, y);
        let x = 0.0;
        let y = 0.1;
        let vel = Vec2Velocity::meters_per_second(x, y) * 1.0;
        commands.spawn().insert_bundle(LocalParticleBundle::new(
            Position(pos),
            Velocity(vel),
            Mass(units::Mass::kilogram(10000000.0)),
        ));
    }

    for _ in 0..n_particles {
        let x = rand::thread_rng().gen_range(4.0..5.0);
        let y = rand::thread_rng().gen_range(-1.0..1.0);
        let pos = Vec2Length::meter(x, y);
        let x = 0.0;
        let y = -0.1;
        let vel = Vec2Velocity::meters_per_second(x, y) * 1.0;
        commands.spawn().insert_bundle(LocalParticleBundle::new(
            Position(pos),
            Velocity(vel),
            Mass(units::Mass::kilogram(10000000.0)),
        ));
    }
}
