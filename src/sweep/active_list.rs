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

    pub fn enumerate_active(
        &self,
        current_level: TimestepLevel,
    ) -> impl Iterator<Item = (&Entity, &T)> {
        self.items
            .iter()
            .filter(move |(_, (level, _))| level.is_active(current_level))
            .map(|(entity, (_, item))| (entity, item))
    }

    pub fn get_mut_and_active_state(
        &mut self,
        entity: Entity,
        current_level: TimestepLevel,
    ) -> (&mut T, bool) {
        let (level, item) = self.items.get_mut(&entity).unwrap();
        (item, level.is_active(current_level))
    }

    pub fn get_mut(&mut self, entity: Entity) -> &mut T {
        &mut self.items.get_mut(&entity).unwrap().1
    }

    pub fn get_mut_with_level(&mut self, entity: Entity) -> (TimestepLevel, &mut T) {
        let (level, item) = self.items.get_mut(&entity).unwrap();
        (*level, item)
    }

    pub fn get(&self, entity: Entity) -> &T {
        &self.items.get(&entity).unwrap().1
    }

    pub fn is_active(&self, entity: Entity, current_level: TimestepLevel) -> bool {
        self.items[&entity].0.is_active(current_level)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.values().map(|(_, item)| item)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.values_mut().map(|(_, item)| item)
    }
}
