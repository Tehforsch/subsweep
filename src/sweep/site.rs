use super::count_by_dir::CountByDir;
use super::timestep_level::TimestepLevel;
use crate::units::Density;
use crate::units::Dimensionless;
use crate::units::PhotonFlux;
use crate::units::SourceRate;

#[derive(Debug)]
pub struct Site {
    pub num_missing_upwind: CountByDir,
    pub flux: Vec<PhotonFlux>,
    pub ionized_hydrogen_fraction: Dimensionless,
    pub density: Density,
    pub source: SourceRate,
    pub absorption_rate: PhotonFlux,
    pub timestep_level: TimestepLevel,
}
