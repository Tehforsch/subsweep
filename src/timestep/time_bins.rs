use std::marker::PhantomData;

use bevy::prelude::*;

pub struct TimeBins<T> {
    num_bins: usize,
    bins: Vec<TimeBin<T>>,
}

impl<T> TimeBins<T> {
    pub fn new(num_bins: usize) -> Self {
        Self {
            num_bins,
            bins: (0..num_bins).map(|_| TimeBin::default()).collect(),
        }
    }

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

impl<T> std::ops::Index<usize> for TimeBins<T> {
    type Output = TimeBin<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.bins[index]
    }
}

impl<T> std::ops::IndexMut<usize> for TimeBins<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.bins[index]
    }
}

pub struct TimeBin<T> {
    _marker: PhantomData<T>,
    particles: Vec<Entity>,
}

impl<T> Default for TimeBin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData::default(),

            particles: vec![],
        }
    }
}

impl<T> TimeBin<T> {
    fn reset(&mut self) {
        self.particles.clear();
    }

    fn insert(&mut self, entity: Entity) {
        self.particles.push(entity);
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Entity> {
        self.particles.iter()
    }
}
