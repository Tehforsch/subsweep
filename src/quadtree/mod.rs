mod config;
mod index;
mod node_index;
mod visualization;

use bevy::prelude::Entity;
pub use config::QuadTreeConfig;
pub use index::QuadTreeIndex;
pub use visualization::QuadTreeVisualizationPlugin;

use crate::domain::extent::Extent;
use crate::physics::MassMoments;
use crate::units::Mass;
use crate::units::VecLength;

pub const MAX_DEPTH: usize = 32;
pub const NUM_DIMENSIONS: usize = 2;
pub const NUM_SUBDIVISIONS: usize = 2usize.pow(NUM_DIMENSIONS as u32);

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

    pub fn make_empty_leaf_from_extent(extent: Extent) -> Self {
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

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::units::DVec2Length;
    use crate::units::Length;

    #[test]
    fn no_infinite_recursion_in_tree_construction_with_close_particles() {
        let positions = [
            LeafData {
                entity: Entity::from_raw(0),
                pos: DVec2Length::meters(1.0, 1.0),
                mass: Mass::zero(),
            },
            LeafData {
                entity: Entity::from_raw(0),
                pos: DVec2Length::meters(1.0, 1.0),
                mass: Mass::zero(),
            },
            LeafData {
                entity: Entity::from_raw(0),
                pos: DVec2Length::meters(2.0, 2.0),
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

    pub fn get_min_depth_quadtree(min_depth: usize) -> QuadTree {
        let positions = [];
        let config = QuadTreeConfig {
            min_depth,
            max_depth: 10,
            ..Default::default()
        };
        let extent = Extent::new(
            Length::meters(0.0),
            Length::meters(1.0),
            Length::meters(0.0),
            Length::meters(1.0),
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
}
