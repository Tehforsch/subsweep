use derive_custom::raxiom_parameters;

use crate::quadtree::QuadTreeConfig;
use crate::units::Dimensionless;
use crate::units::EnergyPerMass;
use crate::units::Length;
use crate::units::Temperature;

/// Parameters for hydrodynamics. Only needed if the
/// [HydrodynamicsPlugin](crate::prelude::HydrodynamicsPlugin)
/// is added to the simulation
#[raxiom_parameters("hydrodynamics")]
pub struct HydrodynamicsParameters {
    /// The number of neighbours that the adaptive smoothing length
    /// tries to aim for.
    pub num_smoothing_neighbours: usize,
    /// The minimum allowed smoothing length.
    pub min_smoothing_length: Length,
    /// The maximum allowed smoothing length.
    pub max_smoothing_length: Length,
    /// How to determine the initial temperature of gas particles.
    pub initial_gas_energy: InitialGasEnergy,
    /// Parameters of the tree used for the neighbour search in the
    /// hydrodynamic density and force calculation. See
    /// [QuadTreeConfig](crate::quadtree::QuadTreeConfig)
    #[serde(default = "default_hydro_tree")]
    pub tree: QuadTreeConfig,
}

#[raxiom_parameters]
#[serde(untagged)]
pub enum InitialGasEnergy {
    /// Set the initial thermal energy u of the gas via two parameters:
    /// 1. The initial temperature T_init
    /// 2. The molecular weight mu of the gas.
    /// This will result in a thermal energy of
    /// u = kB T_init / (mu m_p (gamma - 1))
    /// where kB is the Boltzmann constant, m_p is the proton mass
    /// and gamma is the adiabatic index.
    TemperatureAndMolecularWeight {
        temperature: Temperature,
        molecular_weight: Dimensionless,
    },
    /// Specify the initial thermal energy u directly
    Energy(EnergyPerMass),
    /// All thermal energies will be set by initial conditions or a
    /// startup system. Any hydro-particle with uninitialized internal
    /// energy will lead to an error.
    Explicit,
}

fn default_hydro_tree() -> QuadTreeConfig {
    QuadTreeConfig {
        min_depth: 0,
        max_depth: 20,
        max_num_particles_per_leaf: 30,
    }
}
