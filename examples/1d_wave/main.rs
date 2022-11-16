#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use std::f64::consts::PI;

use bevy::prelude::*;
use raxiom::components;
use raxiom::components::Mass;
use raxiom::components::Position;
use raxiom::ics::DensityProfile;
use raxiom::ics::InitialConditionsPlugin;
use raxiom::ics::IntegerTuple;
use raxiom::ics::RegularSampler;
use raxiom::parameters::HydrodynamicsParameters;
use raxiom::parameters::InitialGasEnergy;
use raxiom::parameters::SimulationParameters;
use raxiom::parameters::TimestepParameters;
use raxiom::prelude::*;
use raxiom::quadtree::QuadTreeConfig;
use raxiom::simulation_plugin::stop_simulation_system;
use raxiom::units::Density;
use raxiom::units::Dimensionless;
use raxiom::units::Length;
use raxiom::units::Pressure;
use raxiom::units::Time;
use raxiom::units::VecLength;
use raxiom::units::GAMMA;

// This test follows Stone et al (2008), section 8.1 and Springel
// (2010), section 8.1

const DENSITY: Density = Density::kilogram_per_square_meter(1.0);
const PRESSURE: Pressure = Pressure::newtons_per_square_meter(3.0 / 5.0);
const WAVE_AMPLITUDE: Dimensionless = Dimensionless::dimensionless(1.0e-6);
const BOX_SIZE: Length = Length::meters(1.0);
const WAVELENGTH: Length = Length::meters(1.0);

#[derive(Clone)]
struct Wave;

impl DensityProfile for Wave {
    fn density(&self, _box: &SimulationBox, pos: VecLength) -> Density {
        let wave_factor = (2.0 * PI * pos.x() / WAVELENGTH).sin();
        DENSITY * (1.0 + WAVE_AMPLITUDE * wave_factor)
    }

    fn max_value(&self) -> Density {
        DENSITY * (1.0 + WAVE_AMPLITUDE)
    }
}

fn build_sim(num_particles: usize) -> Simulation {
    let speed_of_sound = (PRESSURE / DENSITY * GAMMA).sqrt();
    let crossing_time = WAVELENGTH / speed_of_sound * 4.0;
    let delta_x = BOX_SIZE / num_particles as Float;
    let max_timestep = 2.5 * delta_x / speed_of_sound;
    let initial_conditions = InitialConditionsPlugin::default()
        .density_profile(Wave)
        .sampler(RegularSampler {
            num_particles_per_dimension: IntegerTuple {
                x: num_particles,
                y: 1,
            },
        });
    let mut sim = Simulation::default();
    sim.add_parameters_explicitly(HydrodynamicsParameters {
        min_smoothing_length: Length::meters(1e-8),
        max_smoothing_length: Length::meters(1.0),
        num_smoothing_neighbours: 20,
        initial_gas_energy: InitialGasEnergy::Explicit,
        tree: QuadTreeConfig::default(),
    })
    .add_parameters_explicitly(SimulationBox::from(Extent::cube_from_side_length(BOX_SIZE)))
    .add_parameters_explicitly(SimulationParameters {
        final_time: Some(crossing_time),
    })
    .add_parameters_explicitly(TimestepParameters {
        num_levels: 1,
        max_timestep,
    });
    SimulationBuilder::new()
        .read_initial_conditions(false)
        .write_output(false)
        .headless(false)
        .parameters_from_relative_path(file!(), "parameters.yml")
        .update_from_command_line_options()
        .build_with_sim(&mut sim)
        .add_startup_system_to_stage(
            SimulationStartupStages::InsertComponents,
            initialize_energy_system,
        )
        .add_system_to_stage(
            CoreStage::PostUpdate,
            check_system.after(stop_simulation_system),
        )
        .add_plugin(HydrodynamicsPlugin)
        .add_plugin(initial_conditions);
    sim
}

fn initialize_energy_system(
    mut commands: Commands,
    rank: Res<WorldRank>,
    mut particles: Particles<(Entity, &Mass)>,
) {
    assert!(rank.is_main(), "This test is only implemented for one rank");
    let num_particles = particles.iter().count();
    for (entity, mass) in particles.iter_mut() {
        let volume = BOX_SIZE.powi::<2>() / num_particles as Float;
        let density = **mass / volume;
        let energy =
            (PRESSURE / density / (GAMMA - 1.0)) * (density / DENSITY).value().powf(GAMMA - 1.0);
        commands.entity(entity).insert((
            components::Pressure::default(),
            components::Density::default(),
            components::SmoothingLength::default(),
            components::InternalEnergy(energy * **mass),
        ));
    }
}

fn check_system(
    particles: Particles<(&Position, &components::Mass)>,
    mut events: EventReader<StopSimulationEvent>,
    time: Res<raxiom::simulation_plugin::Time>,
    box_: Res<SimulationBox>,
) {
    // if !events.iter().next().is_some() {
    //     return;
    // }
    let num_particles = particles.iter().count() as Float;
    let mut l1 = 0.0;
    for (pos, mass) in particles.iter() {
        // After one crossing time, the final state should be the initial state.
        let wave = Wave;
        let desired = wave.density(&box_, **pos);
        let volume = BOX_SIZE.squared() / num_particles;
        let density = **mass / volume;
        l1 += ((density - desired) / DENSITY / num_particles).value();
    }
    println!("{:?} {:?}", **time, l1);
}

fn main() {
    let mut sim = build_sim(100);
    sim.run();
}
