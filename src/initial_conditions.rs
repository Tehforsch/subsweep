use bevy::prelude::Commands;
use bevy::prelude::Res;
use bevy::prelude::*;
use rand::Rng;
use serde::Deserialize;

use crate::communication::WorldRank;
use crate::io::input;
use crate::parameters::ParameterPlugin;
use crate::particle::LocalParticleBundle;
use crate::plugin_utils::get_parameters;
use crate::position::Position;
use crate::units::DVec2Length;
use crate::units::DVec2Velocity;
use crate::units::Mass;
use crate::units::VecLength;
use crate::units::VecVelocity;
use crate::units::GRAVITY_CONSTANT;
use crate::velocity::Velocity;

pub struct InitialConditionsPlugin;

#[derive(Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Parameters {
    Random(usize),
    EarthSun,
    Read(input::Parameters),
    Figure8,
}

impl Parameters {
    pub fn should_read_initial_conditions(&self) -> bool {
        matches!(self, Self::Read(_))
    }

    pub fn unwrap_read(&self) -> &input::Parameters {
        match self {
            Self::Read(parameters) => parameters,
            _ => panic!("Called unwrap_read on other variant"),
        }
    }
}

impl Default for Parameters {
    fn default() -> Self {
        Self::Read(input::Parameters::default())
    }
}

impl Plugin for InitialConditionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ParameterPlugin::<Parameters>::new("initial_conditions"));
        let parameters = get_parameters::<Parameters>(app);
        match parameters {
            Parameters::Random(_) => {
                app.add_startup_system(spawn_particles_system);
            }
            Parameters::EarthSun => {
                app.add_startup_system(spawn_solar_system_system);
            }
            Parameters::Figure8 => {
                app.add_startup_system(spawn_figure_8_system);
            }
            Parameters::Read(_) => {}
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
    let num_particles = match *parameters {
        Parameters::Random(num_particles) => num_particles,
        _ => unreachable!(),
    };
    let mut rng = rand::thread_rng();
    for _ in 0..num_particles {
        let x = rng.gen_range(-1.0..1.0);
        let y = rng.gen_range(-1.0..1.0);
        let pos = 0.10 * DVec2Length::astronomical_units(x, y);
        let x = rng.gen_range(-1.0..1.0);
        let y = rng.gen_range(-1.0..1.0);
        let vel = 0.04 * DVec2Velocity::astronomical_units_per_day(x, y);
        spawn_particle(&mut commands, pos, vel, Mass::solar(0.01))
    }
}

fn spawn_solar_system_system(mut commands: Commands, rank: Res<WorldRank>) {
    if !rank.is_main() {
        return;
    }
    let positions: Vec<VecLength> = vec![
        VecLength::astronomical_units(0.0, 0.0),
        VecLength::astronomical_units(0.7, 0.7),
    ];
    let masses: Vec<Mass> = vec![Mass::solar(1.0), Mass::earth(1.0)];
    let mass_ratio = masses[0] / masses[1];
    let mass_ratio = mass_ratio.value();
    let velocity: Vec<VecVelocity> = vec![
        VecVelocity::astronomical_units_per_day(-1e-2 / mass_ratio, 1e-2 / mass_ratio),
        VecVelocity::astronomical_units_per_day(1e-2, -1e-2),
    ];
    for ((pos, vel), mass) in positions.into_iter().zip(velocity).zip(masses) {
        spawn_particle(&mut commands, pos, vel, mass)
    }
}

fn spawn_figure_8_system(mut commands: Commands, rank: Res<WorldRank>) {
    if !rank.is_main() {
        return;
    }
    let factor = 1.0;
    let x1 = factor * VecLength::meters(0.97000436, -0.24308753);
    let x2 = -x1;
    let x3 = VecLength::zero();
    let v1 = VecVelocity::meters_per_second(0.466203685, 0.43235673);
    let v2 = v1;
    let v3 = -2.0 * v1;
    let gravity_factor = 1.0 / GRAVITY_CONSTANT.unwrap_value();
    spawn_particle(&mut commands, x1, v1, gravity_factor * Mass::kilograms(1.0));
    spawn_particle(&mut commands, x2, v2, gravity_factor * Mass::kilograms(1.0));
    spawn_particle(&mut commands, x3, v3, gravity_factor * Mass::kilograms(1.0));
}
