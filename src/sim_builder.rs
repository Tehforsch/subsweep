use bevy::core::DefaultTaskPoolOptions;
use bevy::ecs::schedule::ReportExecutionOrderAmbiguities;
use bevy::log::Level;
use bevy::log::LogPlugin;
use bevy::log::LogSettings;
use bevy::prelude::debug;
use bevy::prelude::CoreStage;
use bevy::prelude::DefaultPlugins;
use bevy::prelude::MinimalPlugins;
use bevy::prelude::Res;
use bevy::winit::WinitSettings;

use super::command_line_options::CommandLineOptions;
use super::domain::DomainDecompositionPlugin;
use super::physics::PhysicsPlugin;
use super::visualization::VisualizationPlugin;
use crate::io::input::InputPlugin;
use crate::named::Named;
use crate::physics::hydrodynamics::HydrodynamicsPlugin;
use crate::physics::GravityPlugin;
use crate::simulation::Simulation;
use crate::simulation::TenetPlugin;
use crate::stages::SimulationStagesPlugin;

pub fn log_setup(verbosity: usize) -> LogSettings {
    match verbosity {
        0 => LogSettings {
            level: Level::INFO,
            filter: "bevy_ecs::world=info,bevy_app::plugin_group=info,bevy_app::app=info,winit=error,bevy_render=error,naga=error,wgpu=error".to_string(),
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
    debug!("Time: {:.3} s", time.to_value(super::units::Time::seconds));
}

fn build_sim(sim: &mut Simulation, opts: &CommandLineOptions) {
    let task_pool_opts = if let Some(num_worker_threads) = opts.num_worker_threads {
        DefaultTaskPoolOptions::with_num_threads(num_worker_threads)
    } else {
        DefaultTaskPoolOptions::default()
    };
    sim.add_parameters_from_file(&opts.parameter_file_path)
        .insert_resource(task_pool_opts)
        .add_plugin(SimulationStagesPlugin)
        .add_plugin(InputPlugin)
        .add_plugin(PhysicsPlugin)
        .add_plugin(DomainDecompositionPlugin)
        .add_plugin(GravityPlugin)
        .add_plugin(HydrodynamicsPlugin);
    if sim.on_main_rank() {
        sim.insert_resource(log_setup(opts.verbosity));
        if opts.headless {
            sim.add_bevy_plugins(MinimalPlugins)
                .add_bevy_plugin(LogPlugin);
        } else {
            let winit_opts = WinitSettings {
                return_from_run: true,
                ..Default::default()
            };
            sim.insert_resource(winit_opts);
            sim.add_bevy_plugins(DefaultPlugins);
        }
        sim.add_system_to_stage(CoreStage::Update, show_time_system);
    } else {
        sim.add_bevy_plugins(MinimalPlugins);
        #[cfg(feature = "mpi")]
        sim.add_bevy_plugin(LogPlugin);
    }
    if opts.headless {
        // Only show execution order ambiguities when running without render plugins
        sim.insert_resource(ReportExecutionOrderAmbiguities);
    } else {
        sim.add_plugin(VisualizationPlugin);
    }
}

#[cfg(feature = "mpi")]
pub fn main() {
    use clap::Parser;

    use super::communication::MpiWorld;
    use super::communication::SizedCommunicator;
    use super::mpi_log;
    use crate::communication::BaseCommunicationPlugin;
    use crate::communication::MPI_UNIVERSE;

    let opts = CommandLineOptions::parse();
    let world: MpiWorld<usize> = MpiWorld::new(0);
    mpi_log::initialize(world.rank(), world.size());
    let mut sim = Simulation::new();
    sim.add_plugin(BaseCommunicationPlugin::new(world.size(), world.rank()));
    build_sim(&mut sim, &opts);
    sim.run();
    MPI_UNIVERSE.drop();
}

#[cfg(not(feature = "mpi"))]
pub fn main() {
    use clap::Parser;

    use crate::communication::build_local_communication_sim;

    let opts = CommandLineOptions::parse();
    build_local_communication_sim(
        |sim| {
            let opts = CommandLineOptions::parse();
            build_sim(sim, &opts)
        },
        opts.num_threads,
    );
}

#[derive(Default)]
pub struct SimulationPlugin {
    pub visualize: bool,
}

impl Named for SimulationPlugin {
    fn name() -> &'static str {
        "simulation"
    }
}

impl TenetPlugin for SimulationPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_plugin(SimulationStagesPlugin)
            .add_plugin(PhysicsPlugin)
            .add_plugin(DomainDecompositionPlugin);
        if self.visualize {
            sim.add_bevy_plugins(DefaultPlugins)
                .add_plugin(VisualizationPlugin);
        } else {
            sim.add_bevy_plugins(MinimalPlugins);
        }
    }
}
