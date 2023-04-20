use std::error::Error;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;

use crate::parameter_plugin::parameter_file_contents::Override;

#[derive(Debug)]
pub struct ParseParameterOverrideError(String);

impl fmt::Display for ParseParameterOverrideError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for ParseParameterOverrideError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        "failed to parse parameter override"
    }
}

impl FromStr for Override {
    type Err = ParseParameterOverrideError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split: Vec<_> = s.split(':').collect();
        if split.len() != 2 {
            return Err(ParseParameterOverrideError(format!(
                "Expected key and value separated by `:`, found `{s}`",
            )));
        }
        let mut keys: Vec<String> = split[0].split('/').map(|x| x.to_owned()).collect();
        let section = keys.remove(0);
        let value = serde_yaml::from_str(split[1]).unwrap_or_else(|e| panic!("Failed to parse parameter value in command line argument. keys: {:?} value: {}\n{}", &keys, &split[1], e));
        Ok(Override {
            section,
            keys,
            value,
        })
    }
}

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct CommandLineOptions {
    pub parameter_overrides: Vec<Override>,
    #[clap(long)]
    pub parameter_file_path: Option<PathBuf>,
    #[clap(short, parse(from_occurrences))]
    pub verbosity: usize,
    #[clap(long)]
    pub num_worker_threads: Option<usize>,
}
