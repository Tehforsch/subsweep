use std::path::PathBuf;

use serde::Deserialize;

use crate::named::Named;
use crate::simulation::Simulation;
use crate::units::Time;

#[derive(Deserialize, Named)]
#[name = "output"]
pub struct OutputParameters {
    #[serde(default)]
    pub time_between_snapshots: Time,
    #[serde(default)]
    pub time_first_snapshot: Option<Time>,
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,
    #[serde(default = "default_snapshots_dir")]
    snapshots_dir: PathBuf,
    #[serde(default = "default_fields")]
    pub fields: Vec<String>,
    #[serde(default = "default_snapshot_padding")]
    pub snapshot_padding: usize,
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

fn default_fields() -> Vec<String> {
    ["position", "mass", "velocity"]
        .map(|x| x.to_string())
        .to_vec()
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
}
