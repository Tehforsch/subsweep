use raxiom::communication::Rank;
use raxiom::prelude::ParticleId;

use super::UniqueParticleId;

pub struct RemoteIdCache;

impl RemoteIdCache {
    pub fn new() -> Self {
        RemoteIdCache
    }

    pub fn find(&self, id: UniqueParticleId) -> (Rank, ParticleId) {
        todo!()
    }
}
