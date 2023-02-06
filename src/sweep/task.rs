use mpi::traits::Equivalence;

use super::direction::DirectionIndex;
use crate::particle::ParticleId;
use crate::units::PhotonFlux;

#[derive(Debug)]
pub struct Task {
    pub id: ParticleId,
    pub dir: DirectionIndex,
}

#[derive(Debug, Equivalence)]
pub struct FluxData {
    pub id: ParticleId,
    pub dir: DirectionIndex,
    pub flux: PhotonFlux,
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.dir.partial_cmp(&other.dir)
    }
}

impl Eq for Task {}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.dir == other.dir
    }
}
