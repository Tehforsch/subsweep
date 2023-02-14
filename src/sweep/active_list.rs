use bevy::utils::StableHashMap;

use super::timestep_level::TimestepLevel;
use crate::particle::ParticleId;

pub struct ActiveList<T> {
    items: StableHashMap<ParticleId, (TimestepLevel, T)>,
}

impl<T> ActiveList<T> {
    pub fn new(
        map: StableHashMap<ParticleId, T>,
        levels: &StableHashMap<ParticleId, TimestepLevel>,
    ) -> Self {
        Self {
            items: map
                .into_iter()
                .map(|(id, item)| (id, (levels[&id], item)))
                .collect(),
        }
    }

    pub fn enumerate_active(
        &self,
        current_level: TimestepLevel,
    ) -> impl Iterator<Item = (&ParticleId, &T)> {
        self.items
            .iter()
            .filter(move |(_, (level, _))| level.is_active(current_level))
            .map(|(id, (_, item))| (id, item))
    }

    pub fn get_mut_and_active_state(
        &mut self,
        id: ParticleId,
        current_level: TimestepLevel,
    ) -> (&mut T, bool) {
        let (level, item) = self.items.get_mut(&id).unwrap();
        (item, level.is_active(current_level))
    }

    pub fn get_mut(&mut self, id: ParticleId) -> &mut T {
        &mut self.items.get_mut(&id).unwrap().1
    }

    pub fn get_mut_with_level(&mut self, id: ParticleId) -> (TimestepLevel, &mut T) {
        let (level, item) = self.items.get_mut(&id).unwrap();
        (*level, item)
    }

    pub fn get(&self, id: ParticleId) -> &T {
        &self.items.get(&id).unwrap().1
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.values().map(|(_, item)| item)
    }
}
