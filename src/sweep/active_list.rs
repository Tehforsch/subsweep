use std::ops::Index;

use bevy::prelude::Entity;
use bevy::utils::HashMap;

use super::timestep_level::TimestepLevel;

pub struct ActiveList<T> {
    items: HashMap<Entity, (TimestepLevel, T)>,
}

impl<T> ActiveList<T> {
    pub fn new(map: HashMap<Entity, T>, levels: &HashMap<Entity, TimestepLevel>) -> Self {
        Self {
            items: map
                .into_iter()
                .map(|(entity, item)| (entity, (levels[&entity], item)))
                .collect(),
        }
    }

    pub fn iter_active(&self, current_level: TimestepLevel) -> impl Iterator<Item = &T> {
        self.enumerate_active(current_level).map(|(_, cell)| cell)
    }

    pub fn enumerate_active(
        &self,
        current_level: TimestepLevel,
    ) -> impl Iterator<Item = (&Entity, &T)> {
        self.items
            .iter()
            .filter(move |(_, (level, _))| level.is_active(current_level))
            .map(|(entity, (_, cell))| (entity, cell))
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        self.items.get_mut(&entity).map(|(_, t)| t)
    }
}

impl<T> Index<Entity> for ActiveList<T> {
    type Output = T;

    fn index(&self, index: Entity) -> &Self::Output {
        &self.items[&index].1
    }
}
