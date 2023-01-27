use super::count_by_dir::CountByDir;
use super::direction::Directions;
use crate::units::helpers::Float;
use crate::units::Density;
use crate::units::Dimensionless;
use crate::units::PhotonFlux;
use crate::units::SourceRate;

#[derive(Debug)]
pub struct Site {
    pub num_missing_upwind: CountByDir,
    pub incoming_total_flux: Vec<PhotonFlux>,
    pub outgoing_total_flux: Vec<PhotonFlux>,
    pub ionized_hydrogen_fraction: Dimensionless,
    pub density: Density,
    source: SourceRate,
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
            incoming_total_flux: directions.enumerate().map(|_| PhotonFlux::zero()).collect(),
            outgoing_total_flux: directions.enumerate().map(|_| PhotonFlux::zero()).collect(),
        }
    }

    pub fn total_incoming_flux(&self) -> PhotonFlux {
        self.incoming_total_flux.iter().copied().sum()
    }

    pub fn source_per_direction_bin(&self, directions: &Directions) -> SourceRate {
        self.source / directions.len() as Float
    }
}
