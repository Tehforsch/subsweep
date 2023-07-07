use std::path::PathBuf;

use derive_custom::raxiom_parameters;
use raxiom::units::Dimensionless;
use raxiom::units::SourceRate;

pub mod bpass;
pub mod read_grid;
pub mod remap;
pub mod sources;
pub mod unit_reader;

#[raxiom_parameters("postprocess")]
pub struct Parameters {
    pub initial_fraction_ionized_hydrogen: Option<Dimensionless>,
    pub sources: SourceType,
    pub grid: GridParameters,
    /// Folder containing the raxiom snapshots from which to remap abundances and energies.
    /// The remapping will be done using the latest (highest-numbered) subfolder in the folder.
    pub remap_from: Option<PathBuf>,
}

#[derive(Default)]
#[raxiom_parameters]
pub enum GridParameters {
    #[default]
    Construct,
    Read(PathBuf),
}

#[raxiom_parameters]
pub enum SourceType {
    FromIcs(FromIcs),
    SingleSource(SourceRate),
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

#[raxiom_parameters]
pub struct FromIcs {
    escape_fraction: Dimensionless,
}
