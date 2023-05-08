use derive_custom::raxiom_parameters;

use crate::units::Dimensionless;
use crate::units::PhotonRate;
use crate::units::VecDimensionless;

#[raxiom_parameters("sweep")]
pub struct SweepParameters {
    pub directions: DirectionsSpecification,
    pub num_timestep_levels: usize,
    pub significant_rate_treshold: PhotonRate,
    pub timestep_safety_factor: Dimensionless,
    pub check_deadlock: bool,
}

#[raxiom_parameters]
#[serde(untagged)]
pub enum DirectionsSpecification {
    Num(usize),
    Explicit(Vec<VecDimensionless>),
}

impl DirectionsSpecification {
    pub fn num(&self) -> usize {
        match self {
            DirectionsSpecification::Num(num) => *num,
            DirectionsSpecification::Explicit(directions) => directions.len(),
        }
    }
}
