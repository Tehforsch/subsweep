use std::ops::Index;
use std::ops::IndexMut;

use serde::Deserialize;

use super::Extents;
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

type LeafData<L> = (VecLength, L);
type Leaf<L> = Vec<LeafData<L>>;

#[derive(Debug)]
pub enum Node<N, L> {
    Node(Box<[QuadTree<N, L>; 4]>),
    Leaf(Leaf<L>),
}

impl<N, L> Node<N, L> {
    fn make_node(&mut self, children: Box<[QuadTree<N, L>; 4]>) -> Leaf<L> {
        let value = std::mem::replace(self, Node::Node(children));
        if let Self::Leaf(leaf) = value {
            leaf
        } else {
            panic!("make_node called on Node value")
        }
    }
}

#[derive(Debug)]
pub struct QuadTree<N, L> {
    pub node: Node<N, L>,
    pub data: N,
    pub extents: Extents,
}

impl<N: Default, L: Clone> QuadTree<N, L> {
    pub fn new<'a>(config: &QuadTreeConfig, particles: Vec<LeafData<L>>) -> Self {
        let extents = Extents::from_positions(particles.iter().map(|particle| &particle.0))
            .expect("Not enough particles to construct quadtree");
        let mut tree = Self::make_empty_leaf_from_extents(extents);
        for (pos, data) in particles.iter() {
            tree.insert(config, (pos.clone(), data.clone()), 0);
        }
        tree
    }

    fn insert(&mut self, config: &QuadTreeConfig, data: LeafData<L>, depth: usize) {
        if let Node::Leaf(ref mut leaf) = self.node {
            if leaf.is_empty() || depth == config.max_depth {
                leaf.push(data);
                return;
            } else {
                self.subdivide(config, depth);
            }
        }
        if let Node::Node(ref mut children) = self.node {
            let quadrant = &mut children[self.extents.get_quadrant_index(&data.0)];
            quadrant.insert(&config, data, depth + 1);
        }
    }

    fn subdivide(&mut self, config: &QuadTreeConfig, depth: usize) {
        debug_assert!(matches!(self.node, Node::Leaf(_)));
        let quadrants = self.extents.get_quadrants();
        let children = Box::new(quadrants.map(Self::make_empty_leaf_from_extents));
        let particles = self.node.make_node(children);
        for particle in particles.into_iter() {
            self.insert(config, particle, depth);
        }
    }

    fn make_empty_leaf_from_extents(extents: Extents) -> Self {
        Self {
            node: Node::Leaf(vec![]),
            data: N::default(),
            extents,
        }
    }

    pub fn depth_first_map(&self, closure: &mut impl FnMut(&Extents) -> ()) {
        match self.node {
            Node::Node(ref node) => {
                for child in node.iter() {
                    child.depth_first_map(closure);
                }
            }
            Node::Leaf(_) => {
                closure(&self.extents);
            }
        }
    }
}

impl<N, L> Index<usize> for QuadTree<N, L> {
    type Output = QuadTree<N, L>;

    fn index(&self, idx: usize) -> &Self::Output {
        if let Node::Node(ref children) = self.node {
            &children[idx]
        } else {
            panic!("index called on leaf node");
        }
    }
}

impl<N, L> IndexMut<usize> for QuadTree<N, L> {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        if let Node::Node(ref mut children) = self.node {
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
        QuadTree::<(), ()>::new(&config, positions.into_iter().collect());
    }
}
