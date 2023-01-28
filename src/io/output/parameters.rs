use std::path::PathBuf;

use derive_custom::raxiom_parameters;

use crate::named::Named;
use crate::simulation::Simulation;
use crate::units::Time;

/// How to handle the case of an already existing output directory.
#[derive(Default)]
#[raxiom_parameters]
pub enum HandleExistingOutput {
    /// Halt program execution.
    #[default]
    Panic,
    /// Overwrite already existing files if the names match. This can
    /// cause inconsistent states in which, for example, snapshots
    /// with higher numbers are from an older simulation.
    Overwrite,
    /// Delete the existing output folder. This will erase all
    /// data of the previous simulation.
    Delete,
}

/// Parameters for the output of the simulation.
/// Only required if write_output
/// is set in the [SimulationBuilder](crate::prelude::SimulationBuilder)
#[raxiom_parameters("output")]
pub struct OutputParameters {
    /// The time between two subsequent snapshots. If set to zero,
    /// snapshots will be written at every timestep.
    #[serde(default)]
    pub time_between_snapshots: Time,
    /// The time at which the first snapshot is written. If None, the
    /// first snapshot is written at the first timestep.
    #[serde(default)]
    pub time_first_snapshot: Option<Time>,
    /// The directory to which the output is written.
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,
    /// The name of the sub-directory of the output directory
    /// to which the snapshots are written
    #[serde(default = "default_snapshots_dir")]
    pub snapshots_dir: PathBuf,
    /// The name of the sub-directory of the output directory
    /// to which the time series are written
    #[serde(default = "default_time_series_dir")]
    pub time_series_dir: PathBuf,
    /// Names of all the fields that should be written to snapshots.
    /// Can be names of both attributes and datasets. Example value:
    /// ["position", "velocity", "time"]
    #[serde(default = "default_fields")]
    pub fields: Vec<String>,
    /// The number of digits that the snapshot numbers should be
    /// zero-padded to.
    #[serde(default = "default_snapshot_padding")]
    pub snapshot_padding: usize,
    /// The name of the file which contains a copy of parameters used
    /// in the simulation.
    #[serde(default = "default_used_parameters_filename")]
    pub used_parameters_filename: String,
    /// What to do when the output folder already exists.
    #[serde(default)]
    pub handle_existing_output: HandleExistingOutput,
}

fn default_snapshot_padding() -> usize {
    3
}

fn default_output_dir() -> PathBuf {
    "output".into()
}

fn default_snapshots_dir() -> PathBuf {
    "snapshots".into()
}

fn default_time_series_dir() -> PathBuf {
    "time_series".into()
}

fn default_fields() -> Vec<String> {
    ["position", "mass", "velocity"]
        .map(|x| x.to_string())
        .to_vec()
}

fn default_used_parameters_filename() -> String {
    "parameters.yml".into()
}

impl OutputParameters {
    pub fn is_desired_field<T: Named>(sim: &Simulation) -> bool {
        sim.unwrap_resource::<Self>()
            .fields
            .iter()
            .any(|field| field == T::name())
    }

    pub fn snapshot_dir(&self) -> PathBuf {
        self.output_dir.join(&self.snapshots_dir)
    }

    pub fn time_series_dir(&self) -> PathBuf {
        self.output_dir.join(&self.time_series_dir)
    }
}
