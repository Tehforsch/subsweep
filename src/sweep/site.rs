use super::count_by_dir::CountByDir;
use super::direction::Directions;
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
}

impl Site {
    pub fn new(
        directions: &Directions,
        density: Density,
        ionized_hydrogen_fraction: Dimensionless,
        source: SourceRate,
    ) -> Self {
        Self {
            density,
            ionized_hydrogen_fraction,
            source,
            num_missing_upwind: CountByDir::empty(),
            flux: directions.enumerate().map(|_| PhotonFlux::zero()).collect(),
        }
    }

    pub fn total_flux(&self) -> PhotonFlux {
        self.flux.iter().copied().sum()
    }
}
