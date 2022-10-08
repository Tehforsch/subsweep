#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use std::ops::Div;

use bevy::prelude::*;
use rand::Rng;
use raxiom::prelude::*;
use raxiom::units::Force;
use raxiom::units::Length;
use raxiom::units::Mass;
use raxiom::units::Time;
use raxiom::units::VecLength;
use raxiom::units::VecVelocity;
use serde::Deserialize;

#[derive(Debug, Copy, Clone, Component)]
enum ParticleType {
    Red,
    Blue,
}

#[derive(Default, Deserialize, Clone)]
struct Parameters {
    num_particles: usize,
    fake_viscosity_timescale: Time,
    box_size: VecLength,
    x_force: Force,
    y_force_factor: <Force as Div<Length>>::Output,
    y_offset: Length,
    particle_mass: Mass,
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
        .add_plugin(HydrodynamicsPlugin)
        .add_startup_system(spawn_particles_system)
        .add_system(external_force_system)
        .add_system(fake_periodic_boundaries_system.after(external_force_system))
        .add_system(fake_viscosity_system.after(external_force_system))
        .run();
}

fn get_y_offset_of_particle_type(parameters: &Parameters, type_: &ParticleType) -> Length {
    match type_ {
        ParticleType::Red => parameters.y_offset,
        ParticleType::Blue => -parameters.y_offset,
    }
}

fn external_force_system(
    mut particles: Query<(
        &Position,
        &raxiom::prelude::Mass,
        &mut Velocity,
        &ParticleType,
    )>,
    timestep: Res<Timestep>,
    parameters: Res<Parameters>,
) {
    for (pos, mass, mut vel, type_) in particles.iter_mut() {
        let center = VecLength::new_y(get_y_offset_of_particle_type(&parameters, type_));
        let mut acceleration = (center - **pos) * parameters.y_force_factor;
        acceleration.set_x(match type_ {
            ParticleType::Red => parameters.x_force,
            ParticleType::Blue => -parameters.x_force,
        });
        **vel += acceleration / **mass * **timestep;
    }
}

fn fake_viscosity_system(
    mut particles: Query<&mut Velocity>,
    timestep: Res<Timestep>,
    parameters: Res<Parameters>,
) {
    for mut vel in particles.iter_mut() {
        **vel = **vel
            * (-**timestep / parameters.fake_viscosity_timescale)
                .value()
                .exp();
    }
}

fn fake_periodic_boundaries_system(
    mut particles: Query<&mut Position>,
    parameters: Res<Parameters>,
) {
    for mut pos in particles.iter_mut() {
        if pos.x() > parameters.box_size.x() / 2.0 {
            **pos -= VecLength::new(parameters.box_size.x(), Length::zero());
        } else if pos.x() < -parameters.box_size.x() / 2.0 {
            **pos += VecLength::new(parameters.box_size.x(), Length::zero());
        }
    }
}

fn spawn_particles_system(
    mut commands: Commands,
    rank: Res<WorldRank>,
    parameters: Res<Parameters>,
) {
    if !rank.is_main() {
        return;
    }
    let num_particles_per_type = parameters.num_particles / 2;
    let mut rng = rand::thread_rng();
    for type_ in [ParticleType::Red, ParticleType::Blue] {
        for _ in 0..num_particles_per_type {
            let offset = get_y_offset_of_particle_type(&parameters, &type_);
            let x = rng.gen_range(-parameters.box_size.x()..parameters.box_size.x());
            let y = rng.gen_range(-parameters.box_size.y()..parameters.box_size.y()) + offset;
            spawn_particle(
                &mut commands,
                VecLength::new(x, y),
                VecVelocity::zero(),
                parameters.particle_mass,
                type_,
            )
        }
    }
}

fn spawn_particle(
    commands: &mut Commands,
    pos: VecLength,
    vel: VecVelocity,
    mass: Mass,
    type_: ParticleType,
) {
    commands.spawn_bundle((
        LocalParticle,
        Position(pos),
        Velocity(vel),
        raxiom::prelude::Mass(mass),
        type_,
        DrawCircle::from_position_and_color(
            pos,
            match type_ {
                ParticleType::Red => Color::RED,
                ParticleType::Blue => Color::BLUE,
            },
        ),
    ));
}
