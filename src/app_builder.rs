use bevy::core::DefaultTaskPoolOptions;
use bevy::ecs::schedule::ReportExecutionOrderAmbiguities;
use bevy::log::Level;
use bevy::log::LogPlugin;
use bevy::log::LogSettings;
use bevy::prelude::debug;
use bevy::prelude::App;
use bevy::prelude::DefaultPlugins;
use bevy::prelude::MinimalPlugins;
use bevy::prelude::ParallelSystemDescriptorCoercion;
use bevy::prelude::Res;

use super::command_line_options::CommandLineOptions;
use super::communication::BaseCommunicationPlugin;
use super::domain::DomainDecompositionPlugin;
use super::initial_conditions::InitialConditionsPlugin;
use super::parameters::add_parameter_file_contents;
use super::physics::time_system;
use super::physics::PhysicsPlugin;
use super::plugin_utils::is_main_rank;
use super::visualization::VisualizationPlugin;

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

fn show_time_system(time: Res<super::physics::Time>) {
    debug!("Time: {:.3} s", time.to_value(super::units::Time::second));
}

fn build_app(app: &mut App, opts: &CommandLineOptions, size: usize, rank: i32) {
    add_parameter_file_contents(app, &opts.parameter_file_path);
    let task_pool_opts = if let Some(num_worker_threads) = opts.num_worker_threads {
        DefaultTaskPoolOptions::with_num_threads(num_worker_threads)
    } else {
        DefaultTaskPoolOptions::default()
    };
    app.insert_resource(task_pool_opts)
        .add_plugin(BaseCommunicationPlugin::new(size, rank))
        .add_plugin(DomainDecompositionPlugin)
        .add_plugin(PhysicsPlugin)
        .add_plugin(InitialConditionsPlugin);
    if is_main_rank(app) {
        app.insert_resource(log_setup(opts.verbosity));
        if opts.headless {
            app.add_plugins(MinimalPlugins).add_plugin(LogPlugin);
        } else {
            app.add_plugins(DefaultPlugins);
        }
        app.add_system(show_time_system.after(time_system));
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

#[cfg(not(feature = "local"))]
pub fn main() {
    use clap::Parser;

    use super::communication::MpiWorld;
    use super::communication::SizedCommunicator;
    use super::mpi_log;
    use crate::communication::MPI_UNIVERSE;

    let opts = CommandLineOptions::parse();
    let world: MpiWorld<usize> = MpiWorld::new(0);
    mpi_log::initialize(world.rank(), world.size());
    let mut app = App::new();
    build_app(&mut app, &opts, world.size(), world.rank());
    app.run();
    MPI_UNIVERSE.drop();
}

#[cfg(feature = "local")]
pub fn main() {
    use clap::Parser;

    use crate::communication::build_local_communication_app;

    let opts = CommandLineOptions::parse();
    build_local_communication_app(
        |app, num_threads, rank| {
            let opts = CommandLineOptions::parse();
            build_app(app, &opts, num_threads, rank)
        },
        opts.num_threads,
    );
}
