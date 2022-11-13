#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use raxiom::ics::DensityProfile;
use raxiom::ics::InitialConditionsPlugin;
use raxiom::ics::MonteCarloSampler;
use raxiom::ics::VelocityProfile;
use raxiom::prelude::*;
use raxiom::units::Density;
use raxiom::units::VecLength;
use raxiom::units::VecVelocity;
use raxiom::units::Velocity;

#[raxiom_parameters("example")]
struct Parameters {
    num_particles: usize,
    velocity_difference: Velocity,
    top_fluid_density: Density,
    bottom_fluid_density: Density,
}

impl DensityProfile for Parameters {
    fn density(&self, pos: VecLength) -> Density {
        if pos.y().is_positive() {
            self.top_fluid_density
        } else {
            self.bottom_fluid_density
        }
    }

    fn max_value(&self) -> Density {
        self.top_fluid_density.max(self.bottom_fluid_density)
    }
}

impl VelocityProfile for Parameters {
    fn velocity(&self, pos: VecLength) -> VecVelocity {
        if pos.y().is_positive() {
            MVec::X * self.velocity_difference / 2.0
        } else {
            MVec::X * -self.velocity_difference / 2.0
        }
    }
}

fn main() {
    let mut sim = SimulationBuilder::new();
    let mut sim = sim
        .parameters_from_relative_path(file!(), "parameters.yml")
        .read_initial_conditions(false)
        .write_output(false)
        .headless(false)
        .update_from_command_line_options()
        .build();
    let parameters = sim
        .add_parameter_type_and_get_result::<Parameters>()
        .clone();
    sim.add_plugin(HydrodynamicsPlugin)
        .add_plugin(
            InitialConditionsPlugin::default()
                .density_profile(parameters.clone())
                .velocity_profile(parameters.clone())
                .sampler(MonteCarloSampler::num_particles(parameters.num_particles)),
        )
        .run();
}
