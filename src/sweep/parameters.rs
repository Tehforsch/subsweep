use derive_custom::subsweep_parameters;

use crate::units::Dimensionless;
use crate::units::PhotonRate;
use crate::units::Time;
use crate::units::VecDimensionless;

#[subsweep_parameters("sweep")]
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
    #[serde(default = "default_timestep_factor")]
    pub chemistry_timestep_safety_factor: Dimensionless,
    /// Whether to run a deadlock check before each sweep. Potentially
    /// heavy impact on performance, should only be used during
    /// debugging.
    #[serde(default)]
    pub check_deadlock: bool,
    /// If true, temperatures and ionization fractions will always be kept above the
    /// values in the ICS (which makes sense for overdense regions which would be kept
    /// ionized and heated by feedback processes which are not modelled in subsweep).
    #[serde(default = "default_prevent_cooling")]
    pub prevent_cooling: bool,
    /// The number of tasks to solve before sending/receiving
    /// outgoing/incoming fluxes.  Low numbers reduce serial
    /// performance, high numbers can reduce parallel performance
    /// because downstream cores (or central cores) will need to wait
    /// for incoming tasks for too long.
    #[serde(default = "default_num_tasks_to_solve_before_send_receive")]
    pub num_tasks_to_solve_before_send_receive: usize,
    #[serde(default = "default_limit_absorption")]
    pub limit_absorption: bool,
}

#[subsweep_parameters]
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
    false
}

fn default_timestep_factor() -> Dimensionless {
    Dimensionless::percent(10.0)
}

fn default_prevent_cooling() -> bool {
    true
}

pub fn default_num_tasks_to_solve_before_send_receive() -> usize {
    10000
}

fn default_limit_absorption() -> bool {
    true
}
