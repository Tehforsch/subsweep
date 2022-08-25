#![allow(incomplete_features)]
#![feature(generic_const_exprs, adt_const_params)]
// Some or our '*_system' functions have a large number of arguments.
// That is not necessarily a bad thing, as they are auto-provided by bevy.
#![allow(clippy::too_many_arguments)]
// Some of the Query<…> types appear rather complex to clippy, but are actually
// perfectly readable.
#![allow(clippy::type_complexity)]

mod command_line_options;
mod communication;
mod config;
mod domain;
mod initial_conditions;
mod mass;
mod parameters;
mod particle;
mod physics;
mod position;
pub mod units;
mod velocity;
mod visualization;

use bevy::ecs::schedule::ReportExecutionOrderAmbiguities;
use bevy::log::Level;
use bevy::log::LogPlugin;
use bevy::log::LogSettings;
use bevy::prelude::debug;
use bevy::prelude::App;
use bevy::prelude::DefaultPlugins;
use bevy::prelude::MinimalPlugins;
use bevy::prelude::Res;
use command_line_options::CommandLineOptions;
use communication::NumRanks;
use communication::WorldRank;
use domain::DomainDecompositionPlugin;
use initial_conditions::InitialConditionsPlugin;
use parameters::add_parameter_file_contents;
use physics::PhysicsPlugin;
use visualization::VisualizationPlugin;

pub const PARTICLE_VISUALIZATION_EXCHANGE_TAG: i32 = 1337;
pub const PARTICLE_EXCHANGE_TAG: i32 = 1338;
pub const SEGMENT_EXCHANGE_TAG: i32 = 1339;
pub const EXTENT_EXCHANGE_TAG: i32 = 1340;
pub const NUM_PARTICLES_EXCHANGE_TAG: i32 = 1341;

fn log_setup(verbosity: usize) -> LogSettings {
    match verbosity {
        0 => LogSettings {
            level: Level::INFO,
            filter: "bevy_ecs::world=info,bevy_app::plugin_group=info,bevy_app::app=info,winit=error,bevy_render=error,naga=error,wgpu=error".to_string(),
            ..Default::default()
        },
        1 => LogSettings {
            level: Level::DEBUG,
            filter: "bevy_ecs::world=info,bevy_app::plugin_group=info,bevy_app::app=info,winit=error,bevy_render=error,naga=error,wgpu=error".to_string(),
        },
        2 => LogSettings {
            level: Level::DEBUG,
            filter: "bevy_ecs::world=debug,bevy_app::plugin_group=info,bevy_app::app=info,winit=error,bevy_render=error,naga=error,wgpu=error".to_string(),
        },
        3 => LogSettings {
            level: Level::DEBUG,
            ..Default::default()
        },
        4 => LogSettings {
            level: Level::TRACE,
            ..Default::default()
        },
        v => unimplemented!("Unsupported verbosity level: {}", v)
    }
}

fn show_time_system(time: Res<crate::physics::Time>) {
    debug!("Time: {:.3} s", time.0.to_value(crate::units::Time::second));
}

fn build_app(app: &mut App, opts: &CommandLineOptions, size: usize, rank: i32) {
    add_parameter_file_contents(app, &opts.parameter_file_path);
    app.insert_resource(WorldRank(rank))
        .insert_resource(NumRanks(size))
        .add_plugin(DomainDecompositionPlugin)
        .add_plugin(PhysicsPlugin)
        .add_plugin(InitialConditionsPlugin);
    if rank == 0 {
        app.insert_resource(log_setup(opts.verbosity));
        if opts.headless {
            app.add_plugins(MinimalPlugins).add_plugin(LogPlugin);
        } else {
            app.add_plugins(DefaultPlugins);
        }
        app.add_system(show_time_system);
    } else {
        app.add_plugins(MinimalPlugins);
        #[cfg(not(feature = "local"))]
        app.add_plugin(LogPlugin);
    }
    if opts.headless {
        // Only show execution order ambiguities when running without render plugins
        app.insert_resource(ReportExecutionOrderAmbiguities);
    } else {
        app.add_plugin(VisualizationPlugin);
    }
}

#[cfg(feature = "local")]
fn main() {
    use std::iter::once;
    use std::thread;

    use clap::Parser;
    use communication::get_local_communicators;
    use communication::Rank;

    let mut app = App::new();
    let opts = CommandLineOptions::parse();
    let mut handles = vec![];
    for rank in 0..opts.num_threads {
        let sub_app = App::new();
        app.add_sub_app(sub_app);
        if rank == 0 {
            build_app(&mut app, &opts.clone(), opts.num_threads, rank);
        } else {
            let opts = opts.clone();
            handles.push(thread::spawn(move || {
                let mut app = App::new();
                build_app(&mut app, &opts.clone(), opts.num_threads, rank);
            }));
        }
    }
    for handle in handles.into_iter() {
        handle.join().unwrap();
    }
}

#[cfg(not(feature = "local"))]
fn main() {
    use clap::Parser;

    use crate::communication::MpiWorld;
    use crate::communication::SizedCommunicator;

    let opts = CommandLineOptions::parse();
    let world: MpiWorld<usize> = MpiWorld::new(0);
    let mut app = App::new();
    build_app(&mut app, &opts, world.size(), world.rank());
    app.run();
}
