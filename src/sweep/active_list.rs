use super::timestep_level::TimestepLevel;
use crate::hash_map::HashMap;
use crate::particle::ParticleId;

pub struct ActiveList<T> {
    items: HashMap<ParticleId, (TimestepLevel, T)>,
}

impl<T> ActiveList<T> {
    pub fn new(map: HashMap<ParticleId, T>, levels: &HashMap<ParticleId, TimestepLevel>) -> Self {
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

    pub fn enumerate_with_levels(&self) -> impl Iterator<Item = (&ParticleId, TimestepLevel, &T)> {
        self.items.iter().map(|(id, (level, t))| (id, *level, t))
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

    pub(crate) fn update_levels(&mut self, new_levels: &HashMap<ParticleId, TimestepLevel>) {
        assert_eq!(self.items.len(), new_levels.len());
        for (id, level) in new_levels.iter() {
            self.items.get_mut(id).unwrap().0 = *level;
        }
    }
}
