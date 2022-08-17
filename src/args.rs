use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct CommandLineOptions {
    #[cfg(feature = "local")]
    pub num_threads: usize,
}
