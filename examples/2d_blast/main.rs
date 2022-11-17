#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use raxiom::ics::DensityProfile;
use raxiom::ics::InitialConditionsPlugin;
use raxiom::ics::MonteCarloSampler;
use raxiom::prelude::*;
use raxiom::units::Density;
use raxiom::units::Length;
use raxiom::units::VecLength;

#[raxiom_parameters("example")]
struct Parameters {
    num_particles: usize,
    min_density: Density,
    max_density: Density,
    radius: Length,
}

impl DensityProfile for Parameters {
    fn density(&self, box_: &SimulationBox, pos: VecLength) -> Density {
        let distance_to_center = box_.periodic_distance(&box_.center, &pos);
        if distance_to_center < self.radius {
            self.max_density
        } else {
            self.min_density
        }
    }

    fn max_value(&self) -> Density {
        self.max_density.max(self.min_density)
    }
}

fn main() {
    let mut sim = SimulationBuilder::new();
    let mut sim = sim
        .parameters_from_relative_path(file!(), "parameters.yml")
        .read_initial_conditions(false)
        .write_output(true)
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
                .sampler(MonteCarloSampler::num_particles(parameters.num_particles)),
        )
        .run();
}
