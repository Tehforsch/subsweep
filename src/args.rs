use clap::Parser;

#[derive(Parser, Debug, Clone, Copy)]
#[clap(author, version, about, long_about = None)]
pub struct CommandLineOptions {
    #[cfg(feature = "local")]
    pub num_threads: usize,
    #[clap(long)]
    pub visualize: bool,
    #[clap(short, parse(from_occurrences))]
    pub verbosity: usize,
}
