use std::marker::PhantomData;

use bevy::ecs::query::ROQueryItem;
use bevy::ecs::query::WorldQuery;
use bevy::prelude::*;

pub struct TimeBins<T> {
    max_num_bins: usize,
    bins: Vec<TimeBin<T>>,
}

impl<T> TimeBins<T> {
    pub fn reset(&mut self) {
        for bin in self.bins.iter_mut() {
            bin.reset();
        }
    }

    pub fn insert_up_to(&mut self, level: usize, entity: Entity) {
        for bin in self.bins[0..level + 1].iter_mut() {
            bin.insert(entity);
        }
    }
}

pub struct TimeBin<T> {
    _marker: PhantomData<T>,
    particles: Vec<Entity>,
}

impl<T> TimeBin<T> {
    fn reset(&mut self) {
        self.particles.clear();
    }

    fn iter_query<'w, 's, Q, F>(
        &'w self,
        query: &'w Query<'w, 's, Q, F>,
    ) -> impl Iterator<Item = ROQueryItem<Q>> + 'w
    where
        Q: WorldQuery,
        F: WorldQuery,
        's: 'w,
    {
        self.particles.iter().map(move |x| query.get(*x).unwrap())
    }

    fn insert(&mut self, entity: Entity) {
        self.particles.push(entity);
    }
}
