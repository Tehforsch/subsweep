use bevy::prelude::Commands;
use bevy::prelude::Res;
use bevy::prelude::*;
use rand::Rng;
use serde::Deserialize;

use crate::communication::WorldRank;
use crate::parameters::ParameterPlugin;
use crate::particle::LocalParticleBundle;
use crate::plugin_utils::get_parameters;
use crate::position::Position;
use crate::units::DVec2Length;
use crate::units::DVec2Velocity;
use crate::units::Mass;
use crate::units::VecLength;
use crate::units::VecVelocity;
use crate::velocity::Velocity;

pub struct InitialConditionsPlugin;

#[derive(Clone, Default, Deserialize)]
enum InitialConditionType {
    #[default]
    Normal,
    EarthSun,
}

#[derive(Clone, Default, Deserialize)]
struct Parameters {
    r#type: InitialConditionType,
    num_particles: usize,
}

impl Plugin for InitialConditionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ParameterPlugin::<Parameters>::new("initial_conditions"));
        let parameters = get_parameters::<Parameters>(app);
        match parameters.r#type {
            InitialConditionType::Normal => app.add_startup_system(spawn_particles_system),
            InitialConditionType::EarthSun => app.add_startup_system(spawn_solar_system_system),
        };
    }
}

fn spawn_particle(commands: &mut Commands, pos: VecLength, vel: VecVelocity, mass: Mass) {
    commands.spawn().insert_bundle(LocalParticleBundle::new(
        Position(pos),
        Velocity(vel),
        crate::mass::Mass(mass),
    ));
}

fn spawn_particles_system(
    mut commands: Commands,
    parameters: Res<Parameters>,
    rank: Res<WorldRank>,
) {
    if !rank.is_main() {
        return;
    }
    let n_particles = parameters.num_particles / 2;
    for _ in 0..n_particles {
        let x = rand::thread_rng().gen_range(-5.0..-4.0);
        let y = rand::thread_rng().gen_range(-1.0..1.0);
        let pos = DVec2Length::meter(x, y);
        let x = 0.0;
        let y = 0.1;
        let vel = DVec2Velocity::meters_per_second(x, y) * 1.0;
        spawn_particle(&mut commands, pos, vel, Mass::kilogram(10000000.0))
    }

    for _ in 0..n_particles {
        let x = rand::thread_rng().gen_range(4.0..5.0);
        let y = rand::thread_rng().gen_range(-1.0..1.0);
        let pos = DVec2Length::meter(x, y);
        let x = 0.0;
        let y = -0.1;
        let vel = DVec2Velocity::meters_per_second(x, y) * 1.0;
        spawn_particle(&mut commands, pos, vel, Mass::kilogram(10000000.0))
    }
}

fn spawn_solar_system_system(mut commands: Commands, rank: Res<WorldRank>) {
    if !rank.is_main() {
        return;
    }
    let positions: Vec<VecLength> = vec![
        VecLength::astronomical_unit(0.0, 0.0),
        VecLength::astronomical_unit(0.7, 0.7),
    ];
    let velocity: Vec<VecVelocity> = vec![
        VecVelocity::astronomical_unit_per_day(-1e-2, 1e-2),
        VecVelocity::astronomical_unit_per_day(0.0, 0.0),
    ];
    let masses: Vec<Mass> = vec![Mass::solar(1.0), Mass::earth(1.0)];
    for ((pos, vel), mass) in positions.into_iter().zip(velocity).zip(masses) {
        spawn_particle(&mut commands, pos, vel, mass)
    }
}
