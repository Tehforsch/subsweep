use raxiom::hash_map::HashMap;
use raxiom::prelude::ParticleId;

use super::UniqueParticleId;

pub struct IdCache {
    map: HashMap<UniqueParticleId, ParticleId>,
}

impl IdCache {
    pub fn new(map: HashMap<UniqueParticleId, ParticleId>) -> Self {
        IdCache { map }
    }

    pub fn find(&self, id: UniqueParticleId) -> ParticleId {
        *self.map.get(&id).unwrap()
    }
}
