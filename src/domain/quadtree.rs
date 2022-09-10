use std::ops::Index;
use std::ops::IndexMut;

use serde::Deserialize;

use super::Extent;
use crate::units::Mass;
use crate::units::VecLength;

#[derive(Deserialize)]
pub struct QuadTreeConfig {
    pub max_depth: usize,
}

impl Default for QuadTreeConfig {
    fn default() -> Self {
        Self { max_depth: 20 }
    }
}

#[derive(Debug)]
pub struct LeafData {
    mass: Mass,
    pos: VecLength,
}

#[derive(Debug)]
pub struct NodeData;

impl NodeData {
    fn update_with(&mut self, pos: &VecLength, mass: &Mass) {
        todo!()
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

#[derive(Debug)]
pub enum QuadTreeConstructionError {
    NotEnoughParticles,
}

impl QuadTree {
    pub fn new<'a>(
        config: &QuadTreeConfig,
        particles: Vec<(VecLength, Mass)>,
    ) -> Result<Self, QuadTreeConstructionError> {
        let extent = Extent::from_positions(particles.iter().map(|particle| &particle.0))
            .ok_or(QuadTreeConstructionError::NotEnoughParticles)?;
        let mut tree = Self::make_empty_leaf_from_extent(extent);
        for (pos, data) in particles.iter() {
            tree.insert_new(config, pos.clone(), data.clone(), 0);
        }
        Ok(tree)
    }

    fn insert_new(&mut self, config: &QuadTreeConfig, pos: VecLength, mass: Mass, depth: usize) {
        self.data.update_with(&pos, &mass);
        self.insert(config, pos, mass, depth)
    }

    fn insert(&mut self, config: &QuadTreeConfig, pos: VecLength, mass: Mass, depth: usize) {
        if let Node::Leaf(ref mut leaf) = self.node {
            if leaf.is_empty() || depth == config.max_depth {
                leaf.push(LeafData { mass, pos });
                return;
            } else {
                self.subdivide(config, depth);
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
            data: NodeData,
            extent,
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
    use crate::units::Vec2Length;

    #[test]
    fn no_infinite_recursion_in_tree_construction_with_close_particles() {
        let positions = [
            (Vec2Length::meter(1.0, 1.0), ()),
            (Vec2Length::meter(1.0, 1.0), ()),
            (Vec2Length::meter(2.0, 2.0), ()),
        ];
        let config = QuadTreeConfig { max_depth: 10 };
        QuadTree::new(&config, positions.into_iter().collect()).unwrap();
    }
}
