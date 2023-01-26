use bevy::prelude::Entity;

use super::direction::DirectionIndex;

#[derive(Debug)]
pub struct Task {
    pub entity: Entity,
    pub dir: DirectionIndex,
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.dir.partial_cmp(&other.dir)
    }
}

impl Eq for Task {}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.entity == other.entity && self.dir == other.dir
    }
}
