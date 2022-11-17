use bevy::prelude::Commands;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::Resource;
use bevy::MinimalPlugins;

use crate::components;
use crate::components::Position;
use crate::components::Velocity;
use crate::parameters::SimulationBox;
use crate::parameters::SimulationParameters;
use crate::parameters::TimestepParameters;
use crate::prelude::LocalParticle;
use crate::prelude::Particles;
use crate::prelude::SimulationStages;
use crate::simulation::Simulation;
use crate::simulation_plugin::SimulationPlugin;
use crate::units::Energy;
use crate::units::Length;
use crate::units::Mass;
use crate::units::VecForce;
use crate::units::VecLength;
use crate::units::VecVelocity;
use crate::units::GRAVITY_CONSTANT;

fn gravity_force(pos1: VecLength, pos2: VecLength, mass1: Mass, mass2: Mass) -> VecForce {
    let distance_vector = pos1 - pos2;
    let distance = distance_vector.length();
    -distance_vector * GRAVITY_CONSTANT * mass1 * mass2 / distance.cubed()
}

fn potential_energy(pos1: VecLength, pos2: VecLength, mass1: Mass, mass2: Mass) -> Energy {
    -GRAVITY_CONSTANT * mass1 * mass2 / (pos1 - pos2).length()
}

fn spawn_particles_system(mut commands: Commands) {
    let pos1 = VecLength::zero();
    let pos2 = VecLength::astronomical_units(1.0167, 0.0, 0.0); // distance at aphelion
    let mass1 = Mass::solar(1.0);
    let mass2 = Mass::kilograms(5.9722e24); // earth mass
    let vel1 = VecVelocity::zero();
    let vel2 = VecVelocity::kilometers_per_second(0.0, 29.29, 0.0); // velocity at aphelion
    commands.spawn((
        components::Position(pos1),
        components::Velocity(vel1),
        components::Mass(mass1),
        LocalParticle,
    ));
    commands.spawn((
        components::Position(pos2),
        components::Velocity(vel2),
        components::Mass(mass2),
        LocalParticle,
    ));
}

#[derive(Resource)]
struct TotalEnergy(Option<Energy>);

fn force_system(
    mut particles: Particles<(&mut Velocity, &Position, &components::Mass)>,
    parameters: Res<TimestepParameters>,
    mut initial_energy: ResMut<TotalEnergy>,
) {
    let mut iter = particles.iter_mut();
    let (mut vel1, pos1, mass1) = iter.next().unwrap();
    let (mut vel2, pos2, mass2) = iter.next().unwrap();

    let kinetic_energy = |vel: VecVelocity, mass| vel.length().squared() * mass;
    let total_energy = kinetic_energy(**vel1, **mass1)
        + kinetic_energy(**vel2, **mass2)
        + potential_energy(**pos1, **pos2, **mass1, **mass2)
        + potential_energy(**pos2, **pos1, **mass2, **mass1);

    let timestep = parameters.max_timestep;
    let force = gravity_force(**pos1, **pos2, **mass1, **mass2);
    **vel1 += force / **mass1 * timestep;
    **vel2 -= force / **mass2 * timestep;
    if let Some(initial_energy) = initial_energy.0 {
        let diff =
            (initial_energy - total_energy).abs() / (initial_energy.abs() + total_energy.abs());
        assert!(diff.value() < 1e-4);
    } else {
        initial_energy.0 = Some(total_energy);
    }
}

fn build_integration_sim(sim: &mut Simulation) {
    use crate::stages::SimulationStagesPlugin;
    use crate::units::Time;

    sim.add_parameter_file_contents("".into())
        .add_parameters_explicitly(TimestepParameters {
            max_timestep: Time::years(1e-3),
            num_levels: 1,
        })
        .add_parameters_explicitly(SimulationBox::cube_from_side_length_centered(
            Length::astronomical_units(100.0),
        ))
        .add_parameters_explicitly(SimulationParameters {
            final_time: Some(Time::years(1.0)),
        })
        .write_output(false)
        .insert_resource(TotalEnergy(None))
        .add_bevy_plugins(MinimalPlugins)
        .add_plugin(SimulationStagesPlugin)
        .add_plugin(SimulationPlugin)
        .add_startup_system(spawn_particles_system)
        .add_system_to_stage(SimulationStages::ForceCalculation, force_system);
}

#[test]
fn integration() {
    let mut sim = Simulation::test();
    build_integration_sim(&mut sim);
    sim.run();
}
