use std::fs;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use bevy_core::prelude::TaskPoolOptions;
use bevy_ecs::schedule::ReportExecutionOrderAmbiguities;
use clap::Parser;
use derive_custom::subsweep_parameters;
use log::LevelFilter;
use simplelog::ColorChoice;
use simplelog::CombinedLogger;
use simplelog::ConfigBuilder;
use simplelog::LevelPadding;
use simplelog::TermLogger;
use simplelog::TerminalMode;
use simplelog::WriteLogger;
use time::UtcOffset;

use super::command_line_options::CommandLineOptions;
use super::domain::DomainPlugin;
use super::simulation_plugin::SimulationPlugin;
use crate::communication::BaseCommunicationPlugin;
use crate::communication::MPI_UNIVERSE;
use crate::io::output::make_output_dirs;
use crate::io::output::parameters::OutputParameters;
use crate::parameter_plugin::parameter_file_contents::Override;
use crate::prelude::WorldRank;
use crate::prelude::WorldSize;
use crate::simulation::Simulation;

pub struct SimulationBuilder {
    pub num_worker_threads: Option<usize>,
    pub parameter_file_path: Option<PathBuf>,
    pub verbosity: usize,
    pub read_initial_conditions: bool,
    pub write_output: bool,
    pub log: bool,
    pub parameter_overrides: Vec<Override>,
    base_communication: Option<BaseCommunicationPlugin>,
    require_parameter_file: bool,
}

#[subsweep_parameters("logging")]
#[derive(Debug)]
struct LogParameters {
    pub verbosity: Option<usize>,
    pub only_main_rank: Option<bool>,
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
            require_parameter_file: false,
        }
    }
}

impl SimulationBuilder {
    pub fn new() -> Self {
        use crate::communication::MpiWorld;
        use crate::communication::SizedCommunicator;

        let world: MpiWorld<usize> = MpiWorld::new();
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
        self.parameter_file_path(&opts.parameter_file_path);
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

    pub fn require_parameter_file(&mut self, require_parameter_file: bool) -> &mut Self {
        self.require_parameter_file = require_parameter_file;
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
            if self.require_parameter_file {
                panic!("No parameter file given. Use the --params argument to pass one.");
            }
            sim.add_parameter_file_contents("{}".into());
        }
        sim.with_parameter_overrides(self.parameter_overrides.clone());
        sim.read_initial_conditions(self.read_initial_conditions)
            .write_output(self.write_output)
            .maybe_add_plugin(self.base_communication.clone());
        let rank = **sim.get_resource::<WorldRank>().unwrap();
        let world_size = **sim.get_resource::<WorldSize>().unwrap();
        let output_params = sim
            .add_parameter_type_and_get_result::<OutputParameters>()
            .clone();
        self.make_output_dir(rank, &output_params);
        self.log_setup(sim, rank, world_size, &output_params);
        sim.add_plugin(SimulationPlugin)
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
        sim.add_bevy_plugin(bevy_core::CorePlugin {
            task_pool_options: self.task_pool_opts(),
        })
        .add_bevy_plugin(bevy_app::ScheduleRunnerPlugin);
    }

    fn task_pool_opts(&self) -> TaskPoolOptions {
        if let Some(num_worker_threads) = self.num_worker_threads {
            TaskPoolOptions::with_num_threads(num_worker_threads)
        } else {
            TaskPoolOptions::default()
        }
    }

    fn log_setup(
        &self,
        sim: &mut Simulation,
        rank: i32,
        num_ranks: usize,
        output_params: &OutputParameters,
    ) {
        if !self.log {
            return;
        }
        let log_params = sim
            .add_parameter_type_and_get_result::<LogParameters>()
            .clone();
        let output_file = self.get_output_file(output_params, rank, num_ranks);
        let parent_folder = output_file.parent().unwrap();
        fs::create_dir_all(parent_folder)
            .unwrap_or_else(|_| panic!("Failed to create log directory at {:?}", parent_folder));
        let level = self.get_log_level(log_params.verbosity);
        let local = chrono::Local::now();
        let offset = local.offset();
        let config = ConfigBuilder::default()
            .set_level_padding(LevelPadding::Right)
            .set_time_offset(UtcOffset::from_whole_seconds(offset.local_minus_utc()).unwrap())
            .set_thread_level(LevelFilter::Off)
            .build();
        if rank == 0 {
            CombinedLogger::init(vec![
                TermLogger::new(
                    level,
                    config.clone(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                ),
                WriteLogger::new(level, config, File::create(output_file).unwrap()),
            ])
            .unwrap();
        } else if !log_params.only_main_rank.unwrap_or(true) {
            WriteLogger::init(level, config, File::create(output_file).unwrap()).unwrap();
        }
    }

    fn get_log_level(&self, parameter_verbosity: Option<usize>) -> LevelFilter {
        let verbosity = parameter_verbosity
            .map(|verbosity| self.verbosity.max(verbosity))
            .unwrap_or(self.verbosity);
        match verbosity {
            0 => LevelFilter::Info,
            1 => LevelFilter::Debug,
            2 => LevelFilter::Trace,
            v => unimplemented!("Unsupported verbosity level: {}", v),
        }
    }

    fn get_output_file(
        &self,
        output_params: &OutputParameters,
        rank: i32,
        num_ranks: usize,
    ) -> PathBuf {
        let padding = ((num_ranks as f64).log10().floor() as usize) + 1;
        let output_file = format!("logs/rank_{:0padding$}.log", rank, padding = padding);
        output_params.output_dir.join(&output_file).into()
    }

    fn make_output_dir(&self, rank: i32, output_params: &OutputParameters) {
        if rank == 0 {
            make_output_dirs(output_params);
        }
        // We will need the output dir immediately for the log files,
        // so make sure everyone waits for it to be created.
        MPI_UNIVERSE.barrier();
    }
}
