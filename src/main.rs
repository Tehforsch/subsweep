#![feature(generic_const_exprs, adt_const_params)]
// Some or our '*_system' functions have a large number of arguments.
// That is not necessarily a bad thing, as they are auto-provided by bevy.
#![allow(clippy::too_many_arguments)]
// Some of the Query<…> types appear rather complex to clippy, but are actually
// perfectly readable.
#![allow(clippy::type_complexity)]

mod config;
mod mpi_world;
mod position;
pub mod units;
mod visualization;

use crate::units::second;
use crate::units::Time;
use bevy::prelude::{App, DefaultPlugins, MinimalPlugins, StartupStage};
use mpi::topology::Rank;
use mpi_world::MpiWorld;
use visualization::setup_camera_system;
use visualization::spawn_sprites_system;

struct Timestep(Time);

fn initialize_mpi_and_add_world_resource(app: &mut App) -> Rank {
    let mpi_world = MpiWorld::new();
    let rank = mpi_world.rank();
    app.insert_non_send_resource(mpi_world);
    rank
}

fn main() {
    let mut app = App::new();
    let rank = initialize_mpi_and_add_world_resource(&mut app);
    if rank == 0 {
        app.add_plugins(DefaultPlugins)
            .add_startup_system(setup_camera_system)
            .add_startup_system_to_stage(StartupStage::PostStartup, spawn_sprites_system);
    } else {
        app.add_plugins(MinimalPlugins);
    }
    app.insert_resource(Timestep(second(1.0))).run();
}
