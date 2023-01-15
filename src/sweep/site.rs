use bevy::prelude::Component;

use super::count_by_dir::CountByDir;
use crate::units::PhotonFlux;

#[derive(Component, Debug)]
pub struct Site {
    pub num_missing_upwind: CountByDir,
    pub flux: Vec<PhotonFlux>,
}
