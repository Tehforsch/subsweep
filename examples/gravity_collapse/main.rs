#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use bevy::prelude::*;
use rand::Rng;
use raxiom::prelude::*;
use raxiom::units::InverseTime;
use raxiom::units::Mass;
use raxiom::units::VecLength;
use raxiom::units::VecVelocity;
use serde::Deserialize;

#[derive(Default, Deserialize, Clone)]
struct Parameters {
    num_particles: usize,
    box_size: VecLength,
    particle_mass: Mass,
    angular_velocity_factor: InverseTime,
}

// Implementing named myself here because of
// https://github.com/rust-lang/rust/issues/54363
impl Named for Parameters {
    fn name() -> &'static str {
        "example"
    }
}

fn main() {
    let mut sim = SimulationBuilder::new();
    sim.parameters_from_relative_path(file!(), "parameters.yml")
        .read_initial_conditions(false)
        .write_output(false)
        .headless(false)
        .update_from_command_line_options()
        .build()
        .add_parameter_type::<Parameters>()
        .add_startup_system(spawn_particles_system)
        .add_plugin(GravityPlugin)
        .run();
}

fn spawn_particles_system(
    mut commands: Commands,
    rank: Res<WorldRank>,
    parameters: Res<Parameters>,
) {
    if !rank.is_main() {
        return;
    }
    let mut rng = rand::thread_rng();
    let box_size = parameters.box_size;
    for _ in 0..parameters.num_particles {
        let x = rng.gen_range((-box_size.x() / 2.0)..(box_size.x() / 2.0));
        let y = rng.gen_range((-box_size.y() / 2.0)..(box_size.y() / 2.0));
        let pos = VecLength::new(x, y);
        let vel = VecLength::new(-pos.y(), pos.x()) * parameters.angular_velocity_factor;
        spawn_particle(&mut commands, pos, vel, parameters.particle_mass)
    }
}

fn spawn_particle(commands: &mut Commands, pos: VecLength, vel: VecVelocity, mass: Mass) {
    commands.spawn_bundle((
        LocalParticle,
        Position(pos),
        Velocity(vel),
        raxiom::prelude::Mass(mass),
    ));
}
