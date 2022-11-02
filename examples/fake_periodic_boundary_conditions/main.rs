#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use std::ops::Div;

use bevy::prelude::*;
use hdf5::H5Type;
use mpi::traits::Equivalence;
use raxiom::components;
use raxiom::components::Position;
use raxiom::components::Timestep;
use raxiom::components::Velocity;
use raxiom::ics::ConstantDensity;
use raxiom::ics::Resolution;
use raxiom::ics::Sampler;
use raxiom::prelude::*;
use raxiom::units::Density;
use raxiom::units::Force;
use raxiom::units::Length;
use raxiom::units::Time;
use raxiom::units::VecLength;
use raxiom::units::VecVelocity;
use serde::Deserialize;

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut)]
#[repr(transparent)]
struct ParticleType(usize);

impl Named for ParticleType {
    fn name() -> &'static str {
        "particle_type"
    }
}

#[derive(Default, Deserialize, Clone)]
struct Parameters {
    num_particles: usize,
    fake_viscosity_timescale: Time,
    box_size: VecLength,
    x_force: Force,
    y_force_factor: <Force as Div<Length>>::Output,
    y_offset: Length,
    density: Density,
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
        .add_component_no_io::<ParticleType>()
        .add_parameter_type::<Parameters>()
        .add_plugin(HydrodynamicsPlugin)
        .add_startup_system(initial_conditions_system)
        .add_system(external_force_system)
        .add_system(fake_periodic_boundaries_system.after(external_force_system))
        .add_system(fake_viscosity_system.after(external_force_system))
        .run();
}

fn get_y_offset_of_particle_type(parameters: &Parameters, type_: usize) -> Length {
    match type_ {
        0 => parameters.y_offset,
        1 => -parameters.y_offset,
        _ => unreachable!(),
    }
}

fn external_force_system(
    mut particles: Particles<(
        &Position,
        &components::Mass,
        &mut Velocity,
        &ParticleType,
        &Timestep,
    )>,
    parameters: Res<Parameters>,
) {
    for (pos, mass, mut vel, type_, timestep) in particles.iter_mut() {
        let center = VecLength::new_y(get_y_offset_of_particle_type(&parameters, type_.0));
        let mut acceleration = (center - **pos) * parameters.y_force_factor;
        acceleration.set_x(match type_.0 {
            0 => parameters.x_force,
            1 => -parameters.x_force,
            _ => unreachable!(),
        });
        **vel += acceleration / **mass * **timestep;
    }
}

fn fake_viscosity_system(
    mut particles: Particles<(&mut Velocity, &Timestep)>,
    parameters: Res<Parameters>,
) {
    for (mut vel, timestep) in particles.iter_mut() {
        **vel = **vel
            * (-**timestep / parameters.fake_viscosity_timescale)
                .value()
                .exp();
    }
}

fn fake_periodic_boundaries_system(
    mut particles: Particles<&mut Position>,
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

fn initial_conditions_system(
    mut commands: Commands,
    rank: Res<WorldRank>,
    parameters: Res<Parameters>,
) {
    if !rank.is_main() {
        return;
    }
    let num_particles_per_type = parameters.num_particles / 2;
    for type_ in [0, 1] {
        Sampler::new(
            ConstantDensity(parameters.density),
            Extent::new(-parameters.box_size / 2.0, parameters.box_size / 2.0),
            Resolution::NumParticles(num_particles_per_type),
        )
        .sample()
        .spawn_with(&mut commands, |entity_commands, _, _| {
            entity_commands.insert_bundle((Velocity(VecVelocity::zero()), ParticleType(type_)));
        });
    }
}
