#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use std::f64::consts::PI;

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use raxiom::components;
use raxiom::components::IonizedHydrogenFraction;
use raxiom::components::Position;
use raxiom::grid::init_cartesian_grid_system;
use raxiom::prelude::*;
use raxiom::sweep::sweep_system;
use raxiom::sweep::timestep_level::TimestepLevel;
use raxiom::sweep::SweepParameters;
use raxiom::units::Dimensionless;
use raxiom::units::Length;
use raxiom::units::NumberDensity;
use raxiom::units::PhotonFlux;
use raxiom::units::Time;
use raxiom::units::Volume;
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
    .add_system_to_stage(
        SimulationStages::ForceCalculation,
        print_ionization_system.after(sweep_system),
    )
    .add_plugin(SweepPlugin)
    .run();
}

fn initialize_sweep_components_system(
    mut commands: Commands,
    particles: Particles<(Entity, &Position)>,
    parameters: Res<Parameters>,
    sweep_parameters: Res<SweepParameters>,
    box_size: Res<SimulationBox>,
) {
    for (entity, pos) in particles.iter() {
        let level = { TimestepLevel(sweep_parameters.num_timestep_levels - 1) };
        commands.entity(entity).insert((
            components::Density(parameters.number_density * PROTON_MASS),
            components::IonizedHydrogenFraction(parameters.initial_fraction_ionized_hydrogen),
            level,
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

fn print_ionization_system(
    ionization: Query<&IonizedHydrogenFraction>,
    parameters: Res<Parameters>,
    time: Res<raxiom::simulation_plugin::Time>,
) {
    let mut volume = Volume::zero();
    for frac in ionization.iter() {
        volume += **frac * parameters.cell_size.powi::<3>();
    }
    let t_rec = Time::megayears(122.4);
    let st_radius = Length::kiloparsec(6.79);
    let radius = (volume / (4.0 * PI / 3.0)).cbrt();
    info!(
        "Radius of ionized region: {:?} {:?}",
        radius,
        (1.0 - (-**time / t_rec).exp()).cbrt() * st_radius
    );
}
