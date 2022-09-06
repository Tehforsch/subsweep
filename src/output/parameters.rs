use std::path::PathBuf;

use serde::Deserialize;

use crate::units::Time;

#[derive(Deserialize)]
pub struct Parameters {
    pub time_between_snapshots: Time,
    pub time_first_snapshot: Time,
    pub output_dir: PathBuf,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            time_between_snapshots: Time::zero(),
            time_first_snapshot: Time::zero(),
            output_dir: "output".into(),
        }
    }
}
