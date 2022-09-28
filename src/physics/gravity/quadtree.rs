use bevy::prelude::Entity;

use crate::physics::MassMoments;
use crate::quadtree::QuadTreeLeafData;
use crate::quadtree::QuadTreeNodeData;
use crate::quadtree::{self};
use crate::units::Mass;
use crate::units::VecLength;

pub type QuadTree = quadtree::QuadTree<NodeData, LeafData>;

#[derive(Debug, Clone)]
pub struct LeafData {
    pub entity: Entity,
    pub mass: Mass,
    pub pos: VecLength,
}

#[derive(Debug, Default)]
pub struct NodeData {
    pub moments: MassMoments,
}

impl QuadTreeLeafData for LeafData {
    fn pos(&self) -> &VecLength {
        &self.pos
    }
}

impl QuadTreeNodeData<LeafData> for NodeData {
    fn update_with(&mut self, leaf: &LeafData) {
        self.moments.add_mass_at(&leaf.pos, &leaf.mass);
    }
}
