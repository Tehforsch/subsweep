#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use std::f64::consts::PI;

use bevy::prelude::*;
use raxiom::ics::ConstantDensity;
use raxiom::ics::InitialConditionsPlugin;
use raxiom::ics::IntegerTuple;
use raxiom::ics::RegularSampler;
use raxiom::ics::VelocityProfile;
use raxiom::prelude::*;
use raxiom::units::Density;
use raxiom::units::Length;
use raxiom::units::VecLength;
use raxiom::units::VecVelocity;
use raxiom::units::Velocity;

#[raxiom_parameters("example")]
struct Parameters {
    num_particles: usize,
    wave: Wave,
    density: Density,
}

#[raxiom_parameters]
struct Wave {
    max_velocity: Velocity,
    wave_length: Length,
}

impl VelocityProfile for Wave {
    fn velocity(&self, pos: VecLength) -> VecVelocity {
        let wave_factor = pos.x() / self.wave_length * (2.0 * PI);
        MVec::X * (wave_factor.sin() + 1.0) * self.max_velocity
    }
}

fn main() {
    let mut sim = SimulationBuilder::new();
    let initial_conditions = InitialConditionsPlugin::default()
        .density_profile(ConstantDensity(Density::kilogram_per_square_meter(1.0)))
        .velocity_profile(Wave {
            max_velocity: Velocity::meters_per_second(1.0),
            wave_length: Length::meters(1.0),
        })
        .sampler(RegularSampler {
            num_particles_per_dimension: IntegerTuple { x: 100, y: 1 },
        });
    sim.parameters_from_relative_path(file!(), "parameters.yml")
        .read_initial_conditions(false)
        .write_output(false)
        .headless(false)
        .update_from_command_line_options()
        .build()
        .add_parameter_type::<Parameters>()
        .add_plugin(HydrodynamicsPlugin)
        .add_plugin(initial_conditions)
        .run();
}
