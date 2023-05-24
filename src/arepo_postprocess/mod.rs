use std::path::PathBuf;

use derive_custom::raxiom_parameters;
use raxiom::units::Dimensionless;
use raxiom::units::SourceRate;

pub mod bpass;
pub mod read_grid;
pub mod sources;
pub mod unit_reader;

#[raxiom_parameters("postprocess")]
pub struct Parameters {
    pub initial_fraction_ionized_hydrogen: Dimensionless,
    pub sources: SourceType,
    pub grid: GridParameters,
}

#[derive(Default)]
#[raxiom_parameters]
pub enum GridParameters {
    #[default]
    Construct,
    Read(PathBuf),
}

#[derive(Default)]
#[raxiom_parameters]
pub enum SourceType {
    #[default]
    FromIcs,
    SingleSource(SourceRate),
}
