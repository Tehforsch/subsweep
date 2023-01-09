use bevy::utils::HashMap;

use super::direction::DirectionIndex;
use super::direction::Directions;

pub struct TaskCounter {
    count_by_dir: HashMap<DirectionIndex, usize>,
    total: usize,
}

impl TaskCounter {
    pub fn new(directions: &Directions, num_particles: usize) -> Self {
        let count_by_dir: HashMap<_, _> = directions
            .enumerate()
            .map(|(index, _)| (index, num_particles))
            .collect();
        let total_count = count_by_dir.values().sum();
        Self {
            count_by_dir,
            total: total_count,
        }
    }

    pub fn total(&self) -> usize {
        self.total
    }

    pub fn reduce(&mut self, dir: DirectionIndex) {
        self.total -= 1;
        *self.count_by_dir.get_mut(&dir).unwrap() -= 1;
    }
}
