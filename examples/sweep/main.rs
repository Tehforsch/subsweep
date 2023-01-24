#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use raxiom::components;
use raxiom::components::Position;
use raxiom::grid::init_cartesian_grid_system;
use raxiom::prelude::*;
use raxiom::units::Dimensionless;
use raxiom::units::Length;
use raxiom::units::NumberDensity;
use raxiom::units::PhotonFlux;
use raxiom::units::PROTON_MASS;

#[raxiom_parameters("sweep_postprocess")]
struct Parameters {
    cell_size: Length,
    number_density: NumberDensity,
    initial_fraction_ionized_hydrogen: Dimensionless,
    source_strength: PhotonFlux,
}

fn main() {
    let mut sim = SimulationBuilder::new();
    let mut sim = sim
        .parameters_from_relative_path(file!(), "parameters.yml")
        .headless(false)
        .write_output(true)
        .read_initial_conditions(false)
        .update_from_command_line_options()
        .build();
    let parameters = sim
        .add_parameter_type_and_get_result::<Parameters>()
        .clone();
    sim.add_startup_system(move |commands: Commands, box_size: Res<SimulationBox>| {
        init_cartesian_grid_system(commands, box_size, parameters.cell_size)
    })
    .add_startup_system_to_stage(
        SimulationStartupStages::InsertDerivedComponents,
        initialize_sweep_components_system,
    )
    .add_plugin(SweepPlugin)
    .run();
}

fn initialize_sweep_components_system(
    mut commands: Commands,
    particles: Particles<(Entity, &Position)>,
    parameters: Res<Parameters>,
    box_size: Res<SimulationBox>,
) {
    for (entity, _) in particles.iter() {
        commands.entity(entity).insert((
            components::Density(parameters.number_density * PROTON_MASS),
            components::IonizedHydrogenFraction(parameters.initial_fraction_ionized_hydrogen),
        ));
    }
    let closest_entity_to_center = particles
        .iter()
        .min_by_key(|(_, pos)| {
            let dist = ***pos - box_size.center();
            OrderedFloat(dist.length().value_unchecked())
        })
        .map(|(entity, _)| entity)
        .unwrap();
    commands
        .entity(closest_entity_to_center)
        .insert(components::Source(parameters.source_strength));
}

// fn set_desired_timestep_system(
//     mut particles: Particles<(&Position, &mut DesiredTimestep)>,
//     parameters: Res<TimestepParameters>,
//     box_size: Res<SimulationBox>,
// ) {
//     for (pos, mut desired_timestep) in particles.iter_mut() {
//         **desired_timestep = if pos.x() < box_size.center().x() {
//             parameters.min_timestep()
//         } else {
//             parameters.max_timestep
//         }
//     }
// }
