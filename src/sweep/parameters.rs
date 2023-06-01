use derive_custom::raxiom_parameters;

use crate::units::Dimensionless;
use crate::units::PhotonRate;
use crate::units::Time;
use crate::units::VecDimensionless;

#[raxiom_parameters("sweep")]
pub struct SweepParameters {
    /// The number (or concrete list) of directions to use in the
    /// sweep.
    pub directions: DirectionsSpecification,
    /// Number of timestep levels to use (the minimum timestep
    /// is t_max * 2^(-num_timestep_levels))
    pub num_timestep_levels: usize,
    /// Whether to run with periodic boundary conditions.
    pub periodic: bool,
    /// The maximum allowed timestep.
    pub max_timestep: Time,
    /// Whether to rotate the direction bins after every (full) sweep step.
    #[serde(default = "default_rotate_directions")]
    pub rotate_directions: bool,
    #[serde(default)]
    pub significant_rate_threshold: PhotonRate,
    #[serde(default = "default_timestep_factor")]
    pub timestep_safety_factor: Dimensionless,
    /// Whether to run a deadlock check before each sweep. Potentially
    /// heavy impact on performance, should only be used during
    /// debugging.
    #[serde(default)]
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

fn default_rotate_directions() -> bool {
    true
}

fn default_timestep_factor() -> Dimensionless {
    Dimensionless::percent(10.0)
}
