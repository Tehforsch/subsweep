use std::ops::Index;

use bevy::prelude::Entity;
use mpi::traits::Equivalence;
use serde::Deserialize;

use super::Extent;
use crate::physics::MassMoments;
use crate::units::Mass;
use crate::units::VecLength;

pub const MAX_DEPTH: usize = 32;
pub const NUM_DIMENSIONS: usize = 2;
pub const NUM_SUBDIVISIONS: usize = 2usize.pow(NUM_DIMENSIONS as u32);

#[derive(Deserialize)]
pub struct QuadTreeConfig {
    pub min_depth: usize,
    pub max_depth: usize,
    pub max_num_particles_per_leaf: usize,
}

impl Default for QuadTreeConfig {
    fn default() -> Self {
        Self {
            min_depth: 0,
            max_depth: 20,
            max_num_particles_per_leaf: 1,
        }
    }
}

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

impl NodeData {
    fn update_with(&mut self, pos: &VecLength, mass: &Mass) {
        self.moments.add_mass_at(pos, mass);
    }

    pub fn num_particles(&self) -> usize {
        self.moments.count()
    }
}

type Tree = Box<[QuadTree; 4]>;
type Leaf = Vec<LeafData>;

#[derive(Debug)]
pub enum Node {
    Tree(Tree),
    Leaf(Leaf),
}

impl Node {
    fn make_node(&mut self, children: Tree) -> Leaf {
        let value = std::mem::replace(self, Node::Tree(children));
        if let Self::Leaf(leaf) = value {
            leaf
        } else {
            panic!("make_node called on Node value")
        }
    }
}

#[derive(Debug)]
pub struct QuadTree {
    pub node: Node,
    pub data: NodeData,
    pub extent: Extent,
}

impl QuadTree {
    pub fn new<'a>(config: &QuadTreeConfig, particles: Vec<LeafData>, extent: &Extent) -> Self {
        let mut tree = Self::make_empty_leaf_from_extent(extent.clone());
        tree.subdivide_to_depth(&config, config.min_depth);
        for particle in particles.iter() {
            tree.insert_new(config, particle.clone(), 0);
        }
        tree
    }

    fn subdivide_to_depth(&mut self, config: &QuadTreeConfig, depth: usize) {
        if depth > 0 {
            self.subdivide(config, depth);
            if let Node::Tree(ref mut children) = self.node {
                for child in children.iter_mut() {
                    child.subdivide_to_depth(config, depth - 1);
                }
            } else {
                unreachable!()
            }
        }
    }

    fn insert_new(&mut self, config: &QuadTreeConfig, leaf_data: LeafData, depth: usize) {
        self.data.update_with(&leaf_data.pos, &leaf_data.mass);
        self.insert(config, leaf_data, depth)
    }

    fn insert(&mut self, config: &QuadTreeConfig, leaf_data: LeafData, depth: usize) {
        if let Node::Leaf(ref mut leaf) = self.node {
            if depth < config.max_depth && leaf.len() > config.max_num_particles_per_leaf {
                self.subdivide(config, depth);
            } else {
                leaf.push(leaf_data);
                return;
            }
        }
        if let Node::Tree(ref mut children) = self.node {
            let quadrant = &mut children[self.extent.get_quadrant_index(&leaf_data.pos)];
            quadrant.insert_new(&config, leaf_data, depth + 1);
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

    fn make_empty_leaf_from_extent(extent: Extent) -> Self {
        Self {
            node: Node::Leaf(vec![]),
            data: NodeData::default(),
            extent,
        }
    }

    pub fn depth_first_map_leaf<'a>(
        &'a self,
        closure: &mut impl FnMut(&'a Extent, &'a [LeafData]) -> (),
    ) {
        match self.node {
            Node::Tree(ref node) => {
                for child in node.iter() {
                    child.depth_first_map_leaf(closure);
                }
            }
            Node::Leaf(ref leaf) => {
                closure(&self.extent, &leaf);
            }
        }
    }
}

#[derive(Equivalence, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct QuadTreeIndex([u8; MAX_DEPTH]);

impl Default for QuadTreeIndex {
    fn default() -> Self {
        Self([NodeIndex::ThisNode.into(); MAX_DEPTH])
    }
}

impl QuadTreeIndex {
    fn internal_iter_all_at_depth(
        depth: usize,
        mut current_index: QuadTreeIndex,
        current_depth: usize,
    ) -> Box<dyn Iterator<Item = Self>> {
        if current_depth < depth {
            Box::new((0..NUM_SUBDIVISIONS).flat_map(move |num_child| {
                current_index.0[current_depth] = NodeIndex::Child(num_child as u8).into();
                Self::internal_iter_all_at_depth(depth, current_index, current_depth + 1)
            }))
        } else {
            let mut current_index = current_index.clone();
            current_index.0[current_depth] = NodeIndex::ThisNode.into();
            Box::new(std::iter::once(current_index))
        }
    }

    pub fn iter_all_nodes_at_depth(depth: usize) -> Box<dyn Iterator<Item = Self>> {
        Self::internal_iter_all_at_depth(depth, QuadTreeIndex::default(), 0)
    }

    // I implemented this thinking I'd need it immediately but didn't,
    // however this will definitely become useful at some point
    #[allow(dead_code)]
    pub fn belongs_to(&self, other_index: &QuadTreeIndex) -> bool {
        for depth in 0..MAX_DEPTH {
            if let NodeIndex::Child(num1) = other_index.0[depth].into() {
                match self.0[depth].into() {
                    NodeIndex::Child(num2) => {
                        if num1 != num2 {
                            return false;
                        }
                    }
                    NodeIndex::ThisNode => {
                        return false;
                    }
                }
            } else {
                return true;
            }
        }
        panic!("Invalid quad tree index which does not terminate before MAX_DEPTH")
    }
}

#[derive(Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum NodeIndex {
    #[default]
    ThisNode,
    Child(u8),
}

impl From<u8> for NodeIndex {
    fn from(val: u8) -> Self {
        if val == NUM_SUBDIVISIONS as u8 {
            Self::ThisNode
        } else {
            Self::Child(val)
        }
    }
}

impl From<NodeIndex> for u8 {
    fn from(val: NodeIndex) -> Self {
        match val {
            NodeIndex::ThisNode => NUM_SUBDIVISIONS as u8,
            NodeIndex::Child(num) => num,
        }
    }
}

impl Index<&QuadTreeIndex> for QuadTree {
    type Output = QuadTree;

    fn index(&self, idx: &QuadTreeIndex) -> &Self::Output {
        self.index_into_depth(idx, 0)
    }
}

impl QuadTree {
    fn index_into_depth(&self, idx: &QuadTreeIndex, depth: usize) -> &Self {
        match idx.0[depth].into() {
            NodeIndex::ThisNode => self,
            NodeIndex::Child(num) => {
                if let Node::Tree(ref children) = self.node {
                    children[num as usize].index_into_depth(idx, depth + 1)
                } else {
                    panic!("Invalid index");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::Length;
    use crate::units::Vec2Length;

    #[test]
    fn no_infinite_recursion_in_tree_construction_with_close_particles() {
        let positions = [
            LeafData {
                entity: Entity::from_raw(0),
                pos: Vec2Length::meter(1.0, 1.0),
                mass: Mass::zero(),
            },
            LeafData {
                entity: Entity::from_raw(0),
                pos: Vec2Length::meter(1.0, 1.0),
                mass: Mass::zero(),
            },
            LeafData {
                entity: Entity::from_raw(0),
                pos: Vec2Length::meter(2.0, 2.0),
                mass: Mass::zero(),
            },
        ];
        let config = QuadTreeConfig {
            max_depth: 10,
            ..Default::default()
        };
        let extent =
            Extent::from_positions(positions.iter().map(|particle| &particle.pos)).unwrap();
        QuadTree::new(&config, positions.into_iter().collect(), &extent);
    }

    fn get_min_depth_quadtree(min_depth: usize) -> QuadTree {
        let positions = [];
        let config = QuadTreeConfig {
            min_depth,
            max_depth: 10,
            ..Default::default()
        };
        let extent = Extent::new(
            Length::meter(0.0),
            Length::meter(1.0),
            Length::meter(0.0),
            Length::meter(1.0),
        );
        QuadTree::new(&config, positions.into_iter().collect(), &extent)
    }

    #[test]
    fn min_depth_works() {
        for min_depth in 0..5 {
            let tree = get_min_depth_quadtree(min_depth);
            let mut num_nodes = 0;
            let mut count = |_, _| {
                num_nodes += 1;
            };
            tree.depth_first_map_leaf(&mut count);
            assert_eq!(num_nodes, 4usize.pow(min_depth as u32));
        }
    }

    #[test]
    fn quadtree_index() {
        let min_depth = 5;
        let mut tree = get_min_depth_quadtree(min_depth);
        // obtain a list of particles we can add into the quadtree
        // from the centers of all the leaf ectents
        let config = QuadTreeConfig::default();
        let mut particles = vec![];
        tree.depth_first_map_leaf(&mut |extent: &Extent, _| {
            particles.push(extent.center);
        });
        for pos in particles.into_iter() {
            let data = LeafData {
                pos,
                mass: Mass::zero(),
                entity: Entity::from_raw(0),
            };
            tree.insert_new(&config, data, 0);
        }
        for index in QuadTreeIndex::iter_all_nodes_at_depth(min_depth) {
            let tree = &tree[&index];
            if let Node::Leaf(ref leaf) = tree.node {
                assert_eq!(leaf.len(), 1);
            } else {
                panic!("This should be a leaf")
            }
        }
    }

    fn get_quadtree_index(nodes: &[NodeIndex]) -> QuadTreeIndex {
        let mut index = QuadTreeIndex::default();
        for (depth, n) in nodes.iter().enumerate() {
            index.0[depth] = (*n).into();
        }
        index
    }

    #[test]
    fn quadtree_index_belongs_to() {
        use NodeIndex::*;
        let index1 = get_quadtree_index(&[Child(0), Child(1), Child(2)]);
        let index2 = get_quadtree_index(&[Child(0), Child(1), ThisNode]);
        let index3 = get_quadtree_index(&[Child(1), Child(2), Child(3)]);
        assert_eq!(index1.belongs_to(&index1), true);
        assert_eq!(index1.belongs_to(&index2), true);
        assert_eq!(index2.belongs_to(&index1), false);
        assert_eq!(index2.belongs_to(&index2), true);
        assert_eq!(index1.belongs_to(&index3), false);
        assert_eq!(index3.belongs_to(&index1), false);
    }

    #[test]
    fn node_index_from_into() {
        use NodeIndex::*;
        let check = |n: NodeIndex| {
            let to: u8 = n.into();
            let converted: NodeIndex = to.into();
            assert_eq!(n, converted);
        };
        check(ThisNode);
        for i in 0..NUM_SUBDIVISIONS {
            check(Child(i as u8));
        }
    }
}
