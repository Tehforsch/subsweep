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
use communication::Communicator;
use communication::ExchangeCommunicator;
use communication::Identified;
use communication::SizedCommunicator;
use communication::SyncCommunicator;
use initial_conditions::InitialConditionsPlugin;
use parameters::add_parameter_file_contents;
use physics::ParticleExchangeData;
use physics::PhysicsPlugin;
use visualization::remote::ParticleVisualizationExchangeData;
use visualization::VisualizationPlugin;

pub const PARTICLE_VISUALIZATION_EXCHANGE_TAG: i32 = 1337;
pub const PARTICLE_EXCHANGE_TAG: i32 = 1338;

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
    debug!(
        "Time: {:.3} s",
        time.0.to_value(crate::units::f32::Time::second)
    );
}

fn build_and_run_app(
    opts: &CommandLineOptions,
    communicator1: Communicator<ParticleExchangeData>,
    communicator2: Communicator<Identified<ParticleVisualizationExchangeData>>,
) {
    let mut app = App::new();
    let rank = communicator1.rank();
    add_parameter_file_contents(&mut app, &opts.parameter_file_path);
    app.insert_resource(rank)
        .add_plugin(PhysicsPlugin)
        .add_plugin(InitialConditionsPlugin)
        .insert_non_send_resource(ExchangeCommunicator::new(communicator1))
        .insert_non_send_resource(SyncCommunicator::new(communicator2));
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
    }
    if opts.headless {
        // Only show execution order ambiguities when running without render plugins
        app.insert_resource(ReportExecutionOrderAmbiguities);
    } else {
        app.add_plugin(VisualizationPlugin);
    }
    app.run();
}

#[cfg(feature = "local")]
fn main() {
    use std::iter::once;
    use std::thread;

    use clap::Parser;
    use communication::get_local_communicators;
    use communication::Rank;

    let opts = CommandLineOptions::parse();
    let mut communicators1 = get_local_communicators(opts.num_threads);
    let mut communicators2 = get_local_communicators(opts.num_threads);
    let mut handles = vec![];
    for rank in (1..opts.num_threads).chain(once(0)) {
        let communicator1 = communicators1.remove(&(rank as Rank)).unwrap();
        let communicator2 = communicators2.remove(&(rank as Rank)).unwrap();
        if rank == 0 {
            build_and_run_app(&opts.clone(), communicator1, communicator2);
        } else {
            let opts = opts.clone();
            handles.push(thread::spawn(move || {
                build_and_run_app(&opts, communicator1, communicator2);
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

    let opts = CommandLineOptions::parse();
    let world1 = Communicator::<ParticleExchangeData>::new(PARTICLE_EXCHANGE_TAG);
    let world2 = Communicator::<Identified<ParticleVisualizationExchangeData>>::new(
        PARTICLE_VISUALIZATION_EXCHANGE_TAG,
    );
    build_and_run_app(&opts, world1, world2);
}
