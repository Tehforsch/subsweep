use super::timestep_level::TimestepLevel;
use crate::communication::Rank;
use crate::hash_map::HashMap;
use crate::particle::ParticleId;

pub struct ActiveList<T> {
    items: Vec<T>,
    levels: Vec<TimestepLevel>,
    rank: Rank,
}

impl<T> ActiveList<T> {
    pub fn new(
        mut map: HashMap<ParticleId, T>,
        level_map: &HashMap<ParticleId, TimestepLevel>,
    ) -> Self {
        let rank = map.iter().next().unwrap().0.rank;
        assert!(map.keys().all(|id| id.rank == rank));
        let mut items = Vec::with_capacity(map.len());
        let mut levels = Vec::with_capacity(map.len());
        for index in 0..map.len() {
            let id = ParticleId {
                index: index as u32,
                rank,
            };
            let t = map.remove(&id).unwrap();
            let level = level_map[&id];
            items.push(t);
            levels.push(level);
        }
        // Make sure there are no items left.
        assert_eq!(map.len(), 0);
        Self {
            items,
            levels: levels,
            rank,
        }
    }

    fn get_id_from_index(&self, index: usize) -> ParticleId {
        ParticleId {
            rank: self.rank,
            index: index as u32,
        }
    }

    pub fn enumerate_active(
        &self,
        current_level: TimestepLevel,
    ) -> impl Iterator<Item = (ParticleId, &T)> {
        self.levels
            .iter()
            .enumerate()
            .filter(move |(_, level)| level.is_active(current_level))
            .map(|(i, _)| (self.get_id_from_index(i), &self.items[i]))
    }

    pub fn enumerate_with_levels(&self) -> impl Iterator<Item = (ParticleId, TimestepLevel, &T)> {
        self.levels
            .iter()
            .zip(self.items.iter())
            .enumerate()
            .map(|(i, (level, t))| (self.get_id_from_index(i), *level, t))
    }

    pub fn get_mut_and_active_state(
        &mut self,
        id: ParticleId,
        current_level: TimestepLevel,
    ) -> (&mut T, bool) {
        debug_assert!(id.rank == self.rank);
        let item = &mut self.items[id.index as usize];
        let level = &mut self.levels[id.index as usize];
        (item, level.is_active(current_level))
    }

    pub fn get_mut(&mut self, id: ParticleId) -> &mut T {
        debug_assert!(id.rank == self.rank);
        &mut self.items[id.index as usize]
    }

    pub fn get_mut_with_level(&mut self, id: ParticleId) -> (TimestepLevel, &mut T) {
        debug_assert!(id.rank == self.rank);
        let item = &mut self.items[id.index as usize];
        let level = self.levels[id.index as usize];
        (level, item)
    }

    pub fn get(&self, id: ParticleId) -> &T {
        debug_assert!(id.rank == self.rank);
        let item = &self.items[id.index as usize];
        &item
    }

    pub(crate) fn get_level(&self, id: ParticleId) -> TimestepLevel {
        debug_assert!(id.rank == self.rank);
        let level = self.levels[id.index as usize];
        level
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }

    pub(crate) fn update_levels(&mut self, new_levels: &HashMap<ParticleId, TimestepLevel>) {
        assert_eq!(self.items.len(), new_levels.len());
        for (id, level) in new_levels.iter() {
            if id.rank == self.rank {
                self.levels[id.index as usize] = *level;
            }
        }
    }
}
