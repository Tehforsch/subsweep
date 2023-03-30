use super::work::Work;
use crate::prelude::ParticleId;
use crate::quadtree::LeafDataType;
use crate::quadtree::NodeDataType;
use crate::quadtree::{self};
use crate::units::VecLength;

pub type QuadTree = quadtree::QuadTree<NodeData, LeafData>;

#[derive(Debug, Clone)]
pub struct LeafData {
    pub id: ParticleId,
    pub pos: VecLength,
}

#[derive(Debug, Default)]
pub struct NodeData {
    pub work: Work,
}

impl LeafDataType for LeafData {
    fn pos(&self) -> &VecLength {
        &self.pos
    }
}

impl NodeDataType<LeafData> for NodeData {
    fn update_with(&mut self, _leaf: &LeafData) {
        self.work += Work(1.0);
    }
}
