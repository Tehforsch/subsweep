use bevy::prelude::*;

use super::parameters::HydrodynamicsParameters;
use crate::domain::GlobalExtent;
use crate::prelude::Position;
use crate::quadtree::LeafDataType;
use crate::quadtree::NodeDataType;
use crate::quadtree::{self};
use crate::units::Length;
use crate::units::VecLength;

pub type QuadTree = quadtree::QuadTree<NodeData, LeafData>;

#[derive(Debug, Clone)]
pub struct LeafData {
    pub entity: Entity,
    pub pos: VecLength,
    pub smoothing_length: Length,
}

#[derive(Debug, Default)]
pub struct NodeData {
    pub largest_smoothing_length: Length,
}

impl LeafDataType for LeafData {
    fn pos(&self) -> &VecLength {
        &self.pos
    }
}

impl NodeDataType<LeafData> for NodeData {
    fn update_with(&mut self, leaf: &LeafData) {
        self.largest_smoothing_length = self.largest_smoothing_length.max(leaf.smoothing_length);
    }
}

pub fn construct_quad_tree_system(
    parameters: Res<HydrodynamicsParameters>,
    particles: Query<(Entity, &Position)>,
    extent: Res<GlobalExtent>,
    mut quadtree: ResMut<QuadTree>,
) {
    let particles: Vec<_> = particles
        .iter()
        .map(|(entity, pos)| LeafData {
            entity,
            pos: pos.0,
            smoothing_length: parameters.smoothing_length,
        })
        .collect();
    *quadtree = QuadTree::new(&parameters.tree, particles, &extent);
}
