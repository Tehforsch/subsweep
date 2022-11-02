#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use bevy::prelude::*;
use raxiom::components;
use raxiom::ics::ConstantDensity;
use raxiom::ics::ResolutionSpecification;
use raxiom::ics::Sampler;
use raxiom::prelude::*;
use raxiom::units::Density;
use raxiom::units::InverseTime;
use raxiom::units::VecLength;
use serde::Deserialize;

#[derive(Default, Deserialize, Clone)]
struct Parameters {
    num_particles: usize,
    box_size: VecLength,
    angular_velocity_factor: InverseTime,
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
        .add_parameter_type::<Parameters>()
        .add_startup_system(initial_conditions_system)
        .add_plugin(GravityPlugin)
        .run();
}

fn initial_conditions_system(
    mut commands: Commands,
    rank: Res<WorldRank>,
    parameters: Res<Parameters>,
) {
    if !rank.is_main() {
        return;
    }
    Sampler::new(
        ConstantDensity(parameters.density),
        Extent::new(-parameters.box_size / 2.0, parameters.box_size / 2.0),
        ResolutionSpecification::NumParticles(parameters.num_particles),
    )
    .sample()
    .spawn_with(&mut commands, |entity_commands, pos: VecLength, _mass| {
        let vel = VecLength::from_xy(-pos.y(), pos.x()) * parameters.angular_velocity_factor;
        entity_commands.insert(components::Velocity(vel));
    });
}
