pub mod config;
mod node_index;
pub mod radius_search;

use bevy_ecs::prelude::Resource;
pub use config::QuadTreeConfig;

use crate::domain::extent::Extent;
use crate::units::VecLength;

#[cfg(feature = "2d")]
pub const NUM_DIMENSIONS: usize = 2;
#[cfg(not(feature = "2d"))]
pub const NUM_DIMENSIONS: usize = 3;

pub const TWO_TO_NUM_DIMENSIONS: usize = 2i32.pow(NUM_DIMENSIONS as u32) as usize;

pub const MAX_DEPTH: usize = 32;

pub trait LeafDataType: Clone {
    fn pos(&self) -> &VecLength;
}

pub trait NodeDataType<L>: Default {
    fn update_with(&mut self, leaf: &L);
}

type Tree<N, L> = Box<[QuadTree<N, L>; TWO_TO_NUM_DIMENSIONS]>;
type Leaf<L> = Vec<L>;

#[derive(Debug)]
pub enum Node<N, L> {
    Tree(Tree<N, L>),
    Leaf(Leaf<L>),
}

impl<N, L> Node<N, L> {
    fn make_node(&mut self, children: Tree<N, L>) -> Leaf<L> {
        let value = std::mem::replace(self, Node::Tree(children));
        if let Self::Leaf(leaf) = value {
            leaf
        } else {
            panic!("make_node called on Node value")
        }
    }

    fn unwrap_tree(&self) -> &[QuadTree<N, L>; TWO_TO_NUM_DIMENSIONS] {
        if let Self::Tree(tree) = self {
            tree
        } else {
            panic!("unwrap_tree called on Tree node")
        }
    }
}

#[derive(Debug, Resource)]
pub struct QuadTree<N, L> {
    pub node: Node<N, L>,
    pub data: N,
    pub extent: Extent,
}

impl<N: NodeDataType<L>, L: LeafDataType> QuadTree<N, L> {
    pub fn new(config: &QuadTreeConfig, particles: Vec<L>, extent: &Extent) -> Self {
        let mut tree = Self::make_empty_leaf_from_extent(extent.clone());
        for particle in particles.iter() {
            tree.insert_new(config, particle.clone(), 0);
        }
        tree
    }

    fn insert_new(&mut self, config: &QuadTreeConfig, leaf_data: L, depth: usize) {
        self.data.update_with(&leaf_data);
        self.insert(config, leaf_data, depth)
    }

    fn insert(&mut self, config: &QuadTreeConfig, leaf_data: L, depth: usize) {
        if let Node::Leaf(ref mut leaf) = self.node {
            if depth < config.max_depth && leaf.len() > config.max_num_particles_per_leaf {
                self.subdivide(config, depth);
            } else {
                leaf.push(leaf_data);
                return;
            }
        }
        if let Node::Tree(ref mut children) = self.node {
            let quadrant = &mut children[self.extent.get_quadrant_index(leaf_data.pos())];
            quadrant.insert_new(config, leaf_data, depth + 1);
        }
    }

    fn subdivide(&mut self, config: &QuadTreeConfig, depth: usize) {
        debug_assert!(matches!(self.node, Node::Leaf(_)));
        let quadrants = self.extent.get_quadrants();
        let children = Box::new(quadrants.map(Self::make_empty_leaf_from_extent));
        let particles = self.node.make_node(children);
        for particle in particles.into_iter() {
            self.insert(config, particle, depth);
        }
    }

    pub fn make_empty_leaf_from_extent(extent: Extent) -> Self {
        Self {
            node: Node::Leaf(vec![]),
            data: N::default(),
            extent,
        }
    }

    pub fn depth_first_map_leaf<'a>(&'a self, closure: &mut impl FnMut(&'a Extent, &'a [L])) {
        match self.node {
            Node::Tree(ref node) => {
                for child in node.iter() {
                    child.depth_first_map_leaf(closure);
                }
            }
            Node::Leaf(ref leaf) => {
                closure(&self.extent, leaf);
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::prelude::MVec;
    use crate::units::Length;

    impl LeafDataType for VecLength {
        fn pos(&self) -> &VecLength {
            self
        }
    }

    impl<T> NodeDataType<T> for () {
        fn update_with(&mut self, _: &T) {}
    }

    #[test]
    fn no_infinite_recursion_in_tree_construction_with_close_particles() {
        let positions = [
            VecLength::from_vector_and_scale(MVec::ONE, Length::meters(1.0)),
            VecLength::from_vector_and_scale(MVec::ONE, Length::meters(1.0)),
            VecLength::from_vector_and_scale(MVec::ONE, Length::meters(2.0)),
        ];
        let config = QuadTreeConfig {
            max_depth: 10,
            ..Default::default()
        };
        let extent = Extent::from_positions(positions.iter()).unwrap();
        QuadTree::<(), VecLength>::new(&config, positions.into_iter().collect(), &extent);
    }
}
