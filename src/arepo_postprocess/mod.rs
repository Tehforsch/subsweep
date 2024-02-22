use std::path::PathBuf;

use derive_custom::subsweep_parameters;
use subsweep::source_systems::Source;
use subsweep::units::Dimensionless;

pub mod luminosity_table;
pub mod read_grid;
pub mod remap;
pub mod sources;
pub mod unit_reader;

#[subsweep_parameters("postprocess")]
pub struct Parameters {
    pub initial_fraction_ionized_hydrogen: Option<Dimensionless>,
    pub sources: SourceType,
    pub grid: GridParameters,
    /// Folder containing the subsweep snapshots from which to remap abundances and energies.
    /// The remapping will be done using the latest (highest-numbered) subfolder in the folder.
    pub remap_from: Option<PathBuf>,
}

#[derive(Default)]
#[subsweep_parameters]
pub enum GridParameters {
    #[default]
    Construct,
    Read(PathBuf),
}

#[subsweep_parameters]
pub enum SourceType {
    Explicit(Vec<Source>),
    FromIcs(FromIcs),
}

impl SourceType {
    pub fn unwrap_from_ics(&self) -> FromIcs {
        if let Self::FromIcs(from_ics) = self {
            from_ics.clone()
        } else {
            panic!()
        }
    }
}

#[subsweep_parameters]
pub struct FromIcs {
    escape_fraction: Option<Dimensionless>,
    /// A path to a table specifying the lookup table of ionizing photon rate
    /// as a function of metallicity and age.
    /// First non-comment line: M Metallicity bins in total metallicity
    /// Second non-comment line: N Age bins in years
    /// Remaining M lines: N entries with the ionizing photon rates in [s^-1 Msol^-1].
    stellar_table: Option<PathBuf>,
    escape_fraction_agn: Option<Dimensionless>,
}
