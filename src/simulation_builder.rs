use std::path::Path;
use std::path::PathBuf;

use bevy::ecs::schedule::ReportExecutionOrderAmbiguities;
use bevy::log::Level;
use bevy::log::LogPlugin;
use bevy::prelude::CorePlugin;
use bevy::prelude::MinimalPlugins;
use bevy::prelude::PluginGroup;
use bevy::prelude::TaskPoolOptions;
use bevy::time::TimePlugin;
use bevy::winit::WinitSettings;
use clap::Parser;

use super::command_line_options::CommandLineOptions;
use super::domain::DomainPlugin;
use super::simulation_plugin::SimulationPlugin;
use crate::communication::BaseCommunicationPlugin;
use crate::parameter_plugin::parameter_file_contents::Override;
use crate::simulation::Simulation;
use crate::stages::SimulationStagesPlugin;

pub struct SimulationBuilder {
    pub num_worker_threads: Option<usize>,
    pub parameter_file_path: Option<PathBuf>,
    pub verbosity: usize,
    pub read_initial_conditions: bool,
    pub write_output: bool,
    pub log: bool,
    pub parameter_overrides: Vec<Override>,
    base_communication: Option<BaseCommunicationPlugin>,
}

impl Default for SimulationBuilder {
    fn default() -> Self {
        Self {
            num_worker_threads: None,
            parameter_file_path: None,
            verbosity: 0,
            read_initial_conditions: true,
            write_output: true,
            log: true,
            base_communication: None,
            parameter_overrides: vec![],
        }
    }
}

impl SimulationBuilder {
    pub fn new() -> Self {
        use crate::communication::MpiWorld;
        use crate::communication::SizedCommunicator;

        let world: MpiWorld<usize> = MpiWorld::new(0);
        crate::mpi_log::initialize(world.rank(), world.size());
        Self {
            base_communication: Some(BaseCommunicationPlugin::new(world.size(), world.rank())),
            ..Default::default()
        }
    }

    pub fn bench() -> Self {
        let mut builder = Self::new();
        builder
            .read_initial_conditions(false)
            .write_output(false)
            .log(false);
        builder
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
        if let Some(num_worker_threads) = opts.num_worker_threads {
            self.num_worker_threads(Some(num_worker_threads));
        }
        if let Some(ref path) = opts.parameter_file_path {
            self.parameter_file_path(path);
        }
        self.verbosity(opts.verbosity);
        self.parameter_overrides = opts.parameter_overrides.clone();
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

    pub fn write_output(&mut self, write_output: bool) -> &mut Self {
        self.write_output = write_output;
        self
    }

    pub fn log(&mut self, log: bool) -> &mut Self {
        self.log = log;
        self
    }

    pub fn build_with_sim<'a>(&self, sim: &'a mut Simulation) -> &'a mut Simulation {
        if let Some(ref file) = self.parameter_file_path {
            sim.add_parameters_from_file(file);
        } else {
            sim.add_parameter_file_contents("{}".into());
        }
        sim.with_parameter_overrides(self.parameter_overrides.clone());
        sim.insert_resource(self.winit_settings())
            .read_initial_conditions(self.read_initial_conditions)
            .write_output(self.write_output)
            .maybe_add_plugin(self.base_communication.clone());
        if sim.on_main_rank() && self.log {
            sim.add_bevy_plugin(self.log_plugin());
        }
        sim.add_plugin(SimulationStagesPlugin)
            .add_plugin(SimulationPlugin)
            .add_plugin(DomainPlugin)
            .insert_resource(ReportExecutionOrderAmbiguities);
        self.add_default_bevy_plugins(sim);
        sim
    }

    pub fn build(&mut self) -> Simulation {
        let mut sim = Simulation::default();
        self.build_with_sim(&mut sim);
        sim
    }

    fn add_default_bevy_plugins(&self, sim: &mut Simulation) {
        sim.add_bevy_plugins(
            MinimalPlugins
                .build()
                .disable::<TimePlugin>()
                .set(CorePlugin {
                    task_pool_options: self.task_pool_opts(),
                }),
        );
    }

    fn task_pool_opts(&self) -> TaskPoolOptions {
        if let Some(num_worker_threads) = self.num_worker_threads {
            TaskPoolOptions::with_num_threads(num_worker_threads)
        } else {
            TaskPoolOptions::default()
        }
    }

    fn winit_settings(&self) -> WinitSettings {
        WinitSettings {
            return_from_run: true,
            ..Default::default()
        }
    }

    fn log_plugin(&self) -> LogPlugin {
        match self.verbosity {
            0 => LogPlugin {
                level: Level::INFO,
                filter: "bevy_ecs::world=info,bevy_app::plugin_group=info,bevy_app::app=info,winit=error,bevy_render=error,naga=error,wgpu=error".to_string(),
            },
            1 => LogPlugin {
                level: Level::DEBUG,
                filter: "bevy_ecs::world=info,bevy_app::plugin_group=info,bevy_app::app=info,winit=error,bevy_render=error,naga=error,wgpu=error".to_string(),
            },
            2 => LogPlugin {
                level: Level::DEBUG,
                filter: "bevy_ecs::world=debug,bevy_app::plugin_group=info,bevy_app::app=info,winit=error,bevy_render=error,naga=error,wgpu=error".to_string(),
            },
            3 => LogPlugin {
                level: Level::DEBUG,
                ..Default::default()
            },
            4 => LogPlugin {
                level: Level::TRACE,
                ..Default::default()
            },
            v => unimplemented!("Unsupported verbosity level: {}", v)
        }
    }
}
