#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use bevy::prelude::*;
use raxiom::ics::DensityProfile;
use raxiom::ics::Resolution;
use raxiom::ics::Sampler;
use raxiom::ics::VelocityProfile;
use raxiom::prelude::*;
use raxiom::units::Density;
use raxiom::units::VecLength;
use raxiom::units::VecVelocity;

#[raxiom_parameters("example")]
struct Parameters {
    num_particles: usize,
    top_fluid: FluidSpecification,
    bottom_fluid: FluidSpecification,
}

#[raxiom_parameters]
#[derive(Copy)]
struct FluidSpecification {
    density: Density,
    initial_velocity: units::Velocity,
}

impl DensityProfile for FluidSpecification {
    fn density(&self, _pos: VecLength) -> Density {
        self.density
    }

    fn max_value(&self) -> Density {
        self.density
    }
}

impl VelocityProfile for FluidSpecification {
    fn velocity(&self, _pos: VecLength) -> VecVelocity {
        MVec::X * self.initial_velocity
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
        .add_startup_system(initial_conditions_system)
        .run();
}

fn initial_conditions_system(
    mut commands: Commands,
    rank: Res<WorldRank>,
    parameters: Res<Parameters>,
    box_size: Res<SimulationBox>,
) {
    if !rank.is_main() {
        return;
    }
    let num_particles_per_fluid = parameters.num_particles / 2;
    let center_left = VecLength::new(-box_size.min.x(), box_size.center.y());
    let center_right = VecLength::new(box_size.max.x(), box_size.center.y());
    let extents = [
        Extent::new(-box_size.min, center_right),
        Extent::new(center_left, box_size.max),
    ];
    let fluids = [parameters.bottom_fluid, parameters.top_fluid];
    for (extent, fluid) in extents.into_iter().zip(fluids) {
        Sampler::new(
            fluid,
            &extent.into(),
            Resolution::NumParticles(num_particles_per_fluid),
        )
        .velocity_profile(fluid)
        .spawn(&mut commands)
    }
}
