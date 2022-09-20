#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use std::path::Path;

use bevy::prelude::*;
use rand::Rng;
use serde::Deserialize;
use tenet::prelude::*;
use tenet::units;
use tenet::units::InverseTimeSquared;
use tenet::units::Length;
use tenet::units::Mass;
use tenet::units::VecLength;
use tenet::units::VecVelocity;

#[derive(Default, Deref, DerefMut, Debug)]
struct GravityCenter(VecLength);

#[derive(Deref, DerefMut, Component)]
struct ParticleType(usize);

#[derive(Default, Deserialize, Clone)]
struct ViscousGravityParameters {
    num_particles: usize,
    fake_gravity_factor: InverseTimeSquared,
    fake_viscosity_timescale: units::Time,
}

fn main() {
    let mut sim = SimulationBuilder::mpi();
    sim.parameter_file_path(Path::new("examples/viscous_hydro/parameters.yml"))
        .headless(false)
        .read_initial_conditions(false)
        .build()
        .add_parameter_type::<ViscousGravityParameters>("viscous_gravity")
        .add_plugin(HydrodynamicsPlugin)
        .add_startup_system(spawn_particles_system)
        .add_system(set_gravity_center_system)
        .add_system(fake_gravity_system.after(set_gravity_center_system))
        .add_system(fake_viscosity_system.after(fake_gravity_system))
        .insert_resource(GravityCenter::default())
        .run();
}

fn fake_gravity_system(
    mut particles: Query<(&Position, &mut Velocity, &ParticleType)>,
    timestep: Res<Timestep>,
    center: Res<GravityCenter>,
    parameters: Res<ViscousGravityParameters>,
) {
    for (pos, mut vel, type_) in particles.iter_mut() {
        let center = match type_.0 {
            0 => **center,
            1 => VecLength::zero(),
            _ => unreachable!(),
        };
        let acceleration = (center - **pos) * parameters.fake_gravity_factor;
        **vel += acceleration * **timestep;
    }
}

fn fake_viscosity_system(
    mut particles: Query<&mut Velocity>,
    timestep: Res<Timestep>,
    parameters: Res<ViscousGravityParameters>,
) {
    for mut vel in particles.iter_mut() {
        **vel = **vel
            * (-**timestep / parameters.fake_viscosity_timescale)
                .value()
                .exp();
    }
}

fn set_gravity_center_system(
    mut center: ResMut<GravityCenter>,
    mut events_reader_cursor: EventReader<CursorMoved>,
    vis_parameters: Res<Parameters>,
    windows: Res<Windows>,
) {
    if let Some(mouse_event) = events_reader_cursor.iter().next() {
        let screen_pos = mouse_event.position
            - Vec2::new(
                windows.primary().width() as f32 / 2.0,
                windows.primary().height() as f32 / 2.0,
            );
        **center = VecLength::new(
            vis_parameters.camera_zoom * screen_pos.x as f64,
            vis_parameters.camera_zoom * screen_pos.y as f64,
        );
    }
}

fn spawn_particles_system(
    mut commands: Commands,
    rank: Res<WorldRank>,
    parameters: Res<ViscousGravityParameters>,
) {
    if !rank.is_main() {
        return;
    }
    let num_particles_per_type = parameters.num_particles / 2;
    let mut rng = rand::thread_rng();
    for type_ in [0, 1] {
        for _ in 0..num_particles_per_type {
            let x = rng.gen_range(-1.0..1.0);
            let y = rng.gen_range(-1.0..1.0);
            let pos = 0.01 * VecLength::astronomical_units(x, y);
            let vx = rng.gen_range(-1.0..1.0);
            let vy = rng.gen_range(-1.0..1.0);
            let vel = 0.04 * VecVelocity::astronomical_units_per_day(vx, vy);
            spawn_particle(&mut commands, pos, vel, Mass::solar(0.01), type_)
        }
    }
}

fn spawn_particle(
    commands: &mut Commands,
    pos: VecLength,
    vel: VecVelocity,
    mass: Mass,
    type_: usize,
) {
    commands.spawn_bundle((
        LocalParticle,
        Position(pos),
        Velocity(vel),
        tenet::prelude::Mass(mass),
        ParticleType(type_),
        DrawCircle {
            position: pos,
            radius: Length::astronomical_units(0.003),
            color: match type_ {
                0 => Color::RED,
                1 => Color::BLUE,
                _ => unreachable!(),
            },
        },
    ));
}
