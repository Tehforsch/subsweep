use std::path::Path;

use bevy::prelude::*;
use rand::Rng;
use tenet::prelude::*;
use tenet::units::Mass;
use tenet::units::VecLength;
use tenet::units::VecVelocity;

fn main() {
    let mut sim = SimulationBuilder::mpi();
    sim.parameter_file_path(Path::new("examples/viscous_hydro/parameters.yml"))
        .headless(false)
        .read_initial_conditions(false)
        .build()
        .add_plugin(HydrodynamicsPlugin)
        .add_startup_system(spawn_particles_system)
        .run();
}

fn spawn_particles_system(mut commands: Commands, rank: Res<WorldRank>) {
    if !rank.is_main() {
        return;
    }
    let num_particles = 1000;
    let mut rng = rand::thread_rng();
    for _ in 0..num_particles {
        let x = rng.gen_range(-1.0..1.0);
        let y = rng.gen_range(-1.0..1.0);
        let pos = 0.10 * VecLength::astronomical_units(x, y);
        let vx = rng.gen_range(-1.0..1.0);
        let vy = rng.gen_range(-1.0..1.0);
        let vel = 0.04 * VecVelocity::astronomical_units_per_day(vx, vy);
        spawn_particle(&mut commands, pos, vel, Mass::solar(0.01))
    }
}

fn spawn_particle(commands: &mut Commands, pos: VecLength, vel: VecVelocity, mass: Mass) {
    commands.spawn_bundle((
        LocalParticle,
        Position(pos),
        Velocity(vel),
        tenet::prelude::Mass(mass),
    ));
}
