use std::ops::Index;
use std::ops::IndexMut;

use serde::Deserialize;

use super::Extent;
use crate::physics::MassMoments;
use crate::units::Mass;
use crate::units::VecLength;

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

#[derive(Debug)]
pub struct LeafData {
    mass: Mass,
    pos: VecLength,
}

#[derive(Debug, Default)]
pub struct NodeData {
    moments: MassMoments,
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
    pub fn new<'a>(
        config: &QuadTreeConfig,
        particles: Vec<(VecLength, Mass)>,
        extent: &Extent,
    ) -> Self {
        let mut tree = Self::make_empty_leaf_from_extent(extent.clone());
        tree.subdivide_to_depth(&config, config.min_depth);
        for (pos, data) in particles.iter() {
            tree.insert_new(config, pos.clone(), data.clone(), 0);
        }
        tree
    }

    fn subdivide_to_depth(&mut self, config: &QuadTreeConfig, depth: usize) {
        self.subdivide(config, depth);
        if let Node::Tree(ref mut children) = self.node {
            for child in children.iter_mut() {
                if depth > 0 {
                    child.subdivide_to_depth(config, depth - 1);
                }
            }
        } else {
            unreachable!()
        }
    }

    fn insert_new(&mut self, config: &QuadTreeConfig, pos: VecLength, mass: Mass, depth: usize) {
        self.data.update_with(&pos, &mass);
        self.insert(config, pos, mass, depth)
    }

    fn insert(&mut self, config: &QuadTreeConfig, pos: VecLength, mass: Mass, depth: usize) {
        if let Node::Leaf(ref mut leaf) = self.node {
            if depth < config.max_depth && leaf.len() > config.max_num_particles_per_leaf {
                self.subdivide(config, depth);
            } else {
                leaf.push(LeafData { mass, pos });
                return;
            }
        }
        if let Node::Tree(ref mut children) = self.node {
            let quadrant = &mut children[self.extent.get_quadrant_index(&pos)];
            quadrant.insert_new(&config, pos, mass, depth + 1);
        }
    }

    fn subdivide(&mut self, config: &QuadTreeConfig, depth: usize) {
        debug_assert!(matches!(self.node, Node::Leaf(_)));
        let quadrants = self.extent.get_quadrants();
        let children = Box::new(quadrants.map(Self::make_empty_leaf_from_extent));
        let particles = self.node.make_node(children);
        for particle in particles.into_iter() {
            self.insert(config, particle.pos, particle.mass, depth);
        }
    }

    fn make_empty_leaf_from_extent(extent: Extent) -> Self {
        Self {
            node: Node::Leaf(vec![]),
            data: NodeData::default(),
            extent,
        }
    }

    #[cfg(test)]
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

impl Index<usize> for QuadTree {
    type Output = QuadTree;

    fn index(&self, idx: usize) -> &Self::Output {
        if let Node::Tree(ref children) = self.node {
            &children[idx]
        } else {
            panic!("index called on leaf node");
        }
    }
}

impl IndexMut<usize> for QuadTree {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        if let Node::Tree(ref mut children) = self.node {
            &mut children[idx]
        } else {
            panic!("index_mut called on leaf node");
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
            (Vec2Length::meter(1.0, 1.0), Mass::zero()),
            (Vec2Length::meter(1.0, 1.0), Mass::zero()),
            (Vec2Length::meter(2.0, 2.0), Mass::zero()),
        ];
        let config = QuadTreeConfig {
            max_depth: 10,
            ..Default::default()
        };
        let extent = Extent::from_positions(positions.iter().map(|(pos, _)| pos)).unwrap();
        QuadTree::new(&config, positions.into_iter().collect(), &extent);
    }

    #[test]
    fn min_depth_works() {
        let positions = [(Vec2Length::meter(0.5, 0.5), Mass::zero())];
        let min_depth = 0;
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
        let tree = QuadTree::new(&config, positions.into_iter().collect(), &extent);
        let mut num_nodes = 0;
        let mut count = |_, _| {
            num_nodes += 1;
        };
        tree.depth_first_map_leaf(&mut count);
        assert_eq!(num_nodes, 4usize.pow(1 + min_depth as u32));
    }
}
