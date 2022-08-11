#![allow(incomplete_features)]
#![feature(generic_const_exprs, adt_const_params)]
// Some or our '*_system' functions have a large number of arguments.
// That is not necessarily a bad thing, as they are auto-provided by bevy.
#![allow(clippy::too_many_arguments)]
// Some of the Query<…> types appear rather complex to clippy, but are actually
// perfectly readable.
#![allow(clippy::type_complexity)]

mod config;
mod mpi_world;
mod physics;
mod position;
pub mod units;
mod velocity;
mod visualization;

use bevy::prelude::App;
use bevy::prelude::Commands;
use bevy::prelude::DefaultPlugins;
use bevy::prelude::MinimalPlugins;
use mpi_world::initialize_mpi_and_add_world_resource;
use physics::PhysicsPlugin;
use position::Position;
use units::f32::meter;
use units::f32::meters_per_second;
use velocity::Velocity;
use visualization::VisualizationPlugin;

fn spawn_particles_system(mut commands: Commands) {
    for i in -5..5 {
        for j in -5..5 {
            commands
                .spawn()
                .insert(Position(meter(i as f32), meter(j as f32)))
                .insert(Velocity(
                    meters_per_second(j as f32),
                    meters_per_second(-i as f32),
                ));
        }
    }
}

fn main() {
    let mut app = App::new();
    let rank = initialize_mpi_and_add_world_resource(&mut app);
    if rank == 0 {
        app.add_plugins(DefaultPlugins)
            .add_plugin(VisualizationPlugin);
    } else {
        app.add_plugins(MinimalPlugins);
    }
    app.add_plugin(PhysicsPlugin)
        .add_startup_system(spawn_particles_system)
        .run();
}
