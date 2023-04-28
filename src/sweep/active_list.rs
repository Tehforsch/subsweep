use super::timestep_level::TimestepLevel;
use crate::communication::Rank;
use crate::hash_map::HashMap;
use crate::particle::ParticleId;

pub struct ActiveList<T> {
    items: Vec<T>,
    levels: Vec<TimestepLevel>,
    max_num_levels: usize,
    valid: bool,
    bins: Vec<Vec<usize>>,
    rank: Rank,
}

impl<T> ActiveList<T> {
    pub fn new(
        mut map: HashMap<ParticleId, T>,
        max_num_levels: usize,
        initial_level: TimestepLevel,
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
            items.push(t);
            levels.push(initial_level);
        }
        // Make sure there are no items left.
        assert_eq!(map.len(), 0);
        let mut list = Self {
            items,
            levels: levels,
            rank,
            valid: false,
            bins: vec![],
            max_num_levels,
        };
        list.update_bins();
        list
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
        assert!(self.valid);
        self.bins[current_level.0..self.max_num_levels]
            .iter()
            .flat_map(|bin| {
                bin.iter()
                    .map(|i| (self.get_id_from_index(*i), &self.items[*i]))
            })
    }

    pub fn enumerate_with_levels(&self) -> impl Iterator<Item = (ParticleId, TimestepLevel, &T)> {
        self.levels
            .iter()
            .zip(self.items.iter())
            .enumerate()
            .map(|(i, (level, t))| (self.get_id_from_index(i), *level, t))
    }

    pub fn enumerate_with_levels_mut(
        &mut self,
    ) -> impl Iterator<Item = (ParticleId, &mut TimestepLevel, &T)> {
        self.valid = false;
        self.levels
            .iter_mut()
            .zip(self.items.iter())
            .enumerate()
            .map(|(i, (level, t))| {
                (
                    ParticleId {
                        index: i as u32,
                        rank: self.rank,
                    },
                    level,
                    t,
                )
            })
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
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

    pub fn get_level(&self, id: ParticleId) -> TimestepLevel {
        debug_assert!(id.rank == self.rank);
        let level = self.levels[id.index as usize];
        level
    }

    pub fn set_level(&mut self, id: ParticleId, level: TimestepLevel) {
        debug_assert!(id.rank == self.rank);
        self.valid = false;
        self.levels[id.index as usize] = level;
    }

    pub(crate) fn update_bins(&mut self) {
        let mut bins = vec![];
        for _ in 0..self.max_num_levels {
            bins.push(vec![]);
        }
        for (i, level) in self.levels.iter().enumerate() {
            bins[level.0].push(i);
        }
        self.bins = bins;
        self.valid = true;
    }
}
