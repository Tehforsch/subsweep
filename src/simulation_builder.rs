use std::path::Path;
use std::path::PathBuf;

use bevy::core::DefaultTaskPoolOptions;
use bevy::ecs::schedule::ReportExecutionOrderAmbiguities;
use bevy::log::Level;
use bevy::log::LogPlugin;
use bevy::log::LogSettings;
use bevy::prelude::DefaultPlugins;
use bevy::prelude::MinimalPlugins;
use bevy::winit::WinitSettings;
use clap::Parser;

use super::command_line_options::CommandLineOptions;
use super::domain::DomainDecompositionPlugin;
use super::physics::PhysicsPlugin;
use super::visualization::VisualizationPlugin;
use crate::communication::BaseCommunicationPlugin;
use crate::io::input::ShouldReadInitialConditions;
use crate::simulation::Simulation;
use crate::stages::SimulationStagesPlugin;

fn get_command_line_options() -> CommandLineOptions {
    CommandLineOptions::parse()
}

#[cfg(feature = "mpi")]
pub fn main() {
    let opts = get_command_line_options();
    let mut sim = SimulationBuilder::mpi();
    sim.with_command_line_options(&opts).build().run();
}

#[cfg(not(feature = "mpi"))]
pub fn main() {
    use crate::communication::build_local_communication_sim;

    let opts = get_command_line_options();
    build_local_communication_sim(
        |sim| {
            let opts = get_command_line_options();
            SimulationBuilder::default()
                .with_command_line_options(&opts)
                .build_with_sim(sim);
        },
        opts.num_threads,
    );
}

pub struct SimulationBuilder {
    pub headless: bool,
    pub num_worker_threads: Option<usize>,
    pub parameter_file_path: Option<PathBuf>,
    pub verbosity: usize,
    pub read_initial_conditions: bool,
    base_communication: Option<BaseCommunicationPlugin>,
}

impl Default for SimulationBuilder {
    fn default() -> Self {
        Self {
            headless: true,
            num_worker_threads: None,
            parameter_file_path: None,
            verbosity: 0,
            read_initial_conditions: true,
            base_communication: None,
        }
    }
}

impl SimulationBuilder {
    #[cfg(feature = "mpi")]
    pub fn mpi() -> Self {
        use crate::communication::MpiWorld;
        use crate::communication::SizedCommunicator;

        let world: MpiWorld<usize> = MpiWorld::new(0);
        crate::mpi_log::initialize(world.rank(), world.size());
        Self {
            base_communication: Some(BaseCommunicationPlugin::new(world.size(), world.rank())),
            ..Default::default()
        }
    }

    pub fn update_from_command_line_options(&mut self) -> &mut Self {
        self.with_command_line_options(&CommandLineOptions::parse())
    }

    pub fn parameters_from_relative_path(
        &mut self,
        file_path: &str,
        param_file_name: &str,
    ) -> &mut Self {
        self.parameter_file_path(
            &Path::new(file_path)
                .parent()
                .expect("Failed to get parent directory of source file")
                .join(param_file_name),
        )
    }

    pub fn with_command_line_options(&mut self, opts: &CommandLineOptions) -> &mut Self {
        if let Some(headless) = opts.headless {
            self.headless(headless);
        }
        if let Some(num_worker_threads) = opts.num_worker_threads {
            self.num_worker_threads(Some(num_worker_threads));
        }
        if let Some(ref path) = opts.parameter_file_path {
            self.parameter_file_path(path);
        }
        self.verbosity(opts.verbosity);
        self
    }

    pub fn headless(&mut self, headless: bool) -> &mut Self {
        self.headless = headless;
        self
    }

    pub fn num_worker_threads(&mut self, num_worker_threads: Option<usize>) -> &mut Self {
        self.num_worker_threads = num_worker_threads;
        self
    }

    pub fn parameter_file_path(&mut self, parameter_file_path: &Path) -> &mut Self {
        self.parameter_file_path = Some(parameter_file_path.to_owned());
        self
    }

    pub fn verbosity(&mut self, verbosity: usize) -> &mut Self {
        self.verbosity = verbosity;
        self
    }

    pub fn read_initial_conditions(&mut self, read_initial_conditions: bool) -> &mut Self {
        self.read_initial_conditions = read_initial_conditions;
        self
    }

    fn build_with_sim(&self, sim: &mut Simulation) {
        sim.add_parameters_from_file(
            &self
                .parameter_file_path
                .clone()
                .expect("No parameter file path given"),
        )
        .insert_resource(self.task_pool_opts())
        .insert_resource(self.log_setup())
        .insert_resource(self.winit_settings())
        .insert_resource(ShouldReadInitialConditions(self.read_initial_conditions))
        .add_bevy_plugin(LogPlugin)
        .maybe_add_plugin(self.base_communication.clone())
        .add_plugin(SimulationStagesPlugin)
        .add_plugin(PhysicsPlugin)
        .add_plugin(DomainDecompositionPlugin);
        self.add_default_bevy_plugins(sim);
        if self.headless {
            // Only show execution order ambiguities when running without render plugins
            sim.insert_resource(ReportExecutionOrderAmbiguities);
        } else {
            sim.add_plugin(VisualizationPlugin);
        }
    }

    pub fn build(&mut self) -> Simulation {
        let mut sim = Simulation::default();
        self.build_with_sim(&mut sim);
        sim
    }

    fn add_default_bevy_plugins(&self, sim: &mut Simulation) {
        if sim.on_main_rank() {
            if self.headless {
                sim.add_bevy_plugins(MinimalPlugins);
            } else {
                sim.add_bevy_plugins_with(DefaultPlugins, |group| group.disable::<LogPlugin>());
            }
        } else {
            sim.add_bevy_plugins(MinimalPlugins);
        }
    }

    fn task_pool_opts(&self) -> DefaultTaskPoolOptions {
        if let Some(num_worker_threads) = self.num_worker_threads {
            DefaultTaskPoolOptions::with_num_threads(num_worker_threads)
        } else {
            DefaultTaskPoolOptions::default()
        }
    }

    fn winit_settings(&self) -> WinitSettings {
        WinitSettings {
            return_from_run: true,
            ..Default::default()
        }
    }

    fn log_setup(&self) -> LogSettings {
        match self.verbosity {
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
}
