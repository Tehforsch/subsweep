use std::path::PathBuf;

use serde::Deserialize;

use crate::named::Named;
use crate::simulation::Simulation;
use crate::units::Time;

#[derive(Deserialize)]
pub struct Parameters {
    pub time_between_snapshots: Time,
    pub time_first_snapshot: Option<Time>,
    pub output_dir: PathBuf,
    pub fields: Vec<String>,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            time_between_snapshots: Time::zero(),
            time_first_snapshot: None,
            output_dir: "output".into(),
            fields: vec![],
        }
    }
}

impl Parameters {
    pub fn is_desired_field<T: Named>(sim: &Simulation) -> bool {
        sim.unwrap_resource::<Self>()
            .fields
            .iter()
            .any(|field| field == T::name())
    }
}
