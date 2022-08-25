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

use bevy::app::AppLabel;
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
use communication::Rank;
use communication::WorldRank;
use domain::DomainDecompositionPlugin;
use initial_conditions::InitialConditionsPlugin;
use parameters::add_parameter_file_contents;
use physics::PhysicsPlugin;
use visualization::VisualizationPlugin;

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

#[derive(AppLabel)]
enum SubAppRank {
    Rank0,
    Rank1,
    Rank2,
    Rank3,
    Rank4,
}

impl SubAppRank {
    fn from_num(num: usize) -> Self {
        match num {
            0 => Self::Rank0,
            1 => Self::Rank1,
            2 => Self::Rank2,
            3 => Self::Rank3,
            4 => Self::Rank4,
            _ => unimplemented!(),
        }
    }
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
    use clap::Parser;

    let mut app = App::new();
    let opts = CommandLineOptions::parse();
    for rank in 1..opts.num_threads {
        let mut sub_app = App::new();
        build_app(&mut sub_app, &opts, opts.num_threads, rank as Rank);
        app.add_sub_app(SubAppRank::from_num(rank), sub_app, |_, app| app.update());
    }
    build_app(&mut app, &opts, opts.num_threads, 0 as Rank);
    app.run();
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
