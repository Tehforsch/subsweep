use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct CommandLineOptions {
    #[cfg(not(feature = "mpi"))]
    pub num_threads: usize,
    pub parameter_file_path: PathBuf,
    #[clap(long)]
    pub headless: bool,
    #[clap(short, parse(from_occurrences))]
    pub verbosity: usize,
    #[clap(long)]
    pub num_worker_threads: Option<usize>,
}
