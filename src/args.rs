use clap::Parser;
use clap::Subcommand;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct CommandLineOptions {
    #[clap(subcommand)]
    pub run_type: RunType,
}
#[derive(Subcommand, Debug)]
pub enum RunType {
    Mpi,
    Local,
}
