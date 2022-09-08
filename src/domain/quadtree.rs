pub mod insertion_data;

use std::ops::Index;
use std::ops::IndexMut;

use serde::Deserialize;

use self::insertion_data::InsertionData;
use super::Extent;

pub trait NodeDataType<P, L> {
    fn add_new_leaf_data(&mut self, _p: &P, _l: &L) {}
    fn add_to_final_node(&mut self, _p: &P, _l: &L) {}
}

impl<P, L> NodeDataType<P, L> for () {}

#[derive(Deserialize)]
pub struct QuadTreeConfig {
    pub max_depth: usize,
}

impl Default for QuadTreeConfig {
    fn default() -> Self {
        Self { max_depth: 20 }
    }
}

type TreeData<N, P, L> = Box<[QuadTree<N, P, L>; 4]>;
type LeafData<P, L> = Vec<(P, L)>;

#[derive(Debug)]
pub enum Node<N, P, L> {
    Tree(TreeData<N, P, L>),
    Leaf(LeafData<P, L>),
}

impl<N, P, L> Node<N, P, L> {
    fn make_node(&mut self, children: TreeData<N, P, L>) -> LeafData<P, L> {
        let value = std::mem::replace(self, Node::Tree(children));
        if let Self::Leaf(leaf) = value {
            leaf
        } else {
            panic!("make_node called on Node value")
        }
    }
}

#[derive(Debug)]
pub struct QuadTree<N, P, L> {
    pub node: Node<N, P, L>,
    pub data: N,
    pub extents: Extent,
}

impl<N, P, L> QuadTree<N, P, L>
where
    N: Default + NodeDataType<P, L>,
    L: Clone,
    P: Clone + InsertionData,
{
    pub fn new<'a>(extents: &Extent, config: &QuadTreeConfig, particles: Vec<(P, L)>) -> Self {
        let mut tree = Self::make_empty_leaf_from_extents(extents.clone());
        for (pos, data) in particles.iter() {
            tree.insert_new(config, (pos.clone(), data.clone()), 0);
        }
        tree
    }

    fn insert_new(&mut self, config: &QuadTreeConfig, data: (P, L), depth: usize) {
        self.data.add_new_leaf_data(&data.0, &data.1);
        self.insert(config, data, depth)
    }

    fn insert(&mut self, config: &QuadTreeConfig, data: (P, L), depth: usize) {
        if let Node::Leaf(ref mut leaf) = self.node {
            if leaf.is_empty() || depth == config.max_depth {
                leaf.push(data);
                return;
            } else {
                self.subdivide(config, depth);
            }
        }
        if let Node::Tree(ref mut children) = self.node {
            let index = data.0.get_quadrant_index(&self.extents);
            match index {
                Some(index) => {
                    let quadrant = &mut children[index];
                    quadrant.insert_new(&config, data, depth + 1);
                }
                None => self.data.add_to_final_node(&data.0, &data.1),
            }
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

    fn make_empty_leaf_from_extents(extents: Extent) -> Self {
        Self {
            node: Node::Leaf(vec![]),
            data: N::default(),
            extents,
        }
    }

    pub fn depth_first_map(&self, closure: &mut impl FnMut(&Extent, &LeafData<P, L>) -> ()) {
        match self.node {
            Node::Tree(ref node) => {
                for child in node.iter() {
                    child.depth_first_map(closure);
                }
            }
            Node::Leaf(ref leaf) => {
                closure(&self.extents, &leaf);
            }
        }
    }
}

impl<N, P, L> Index<usize> for QuadTree<N, P, L> {
    type Output = QuadTree<N, P, L>;

    fn index(&self, idx: usize) -> &Self::Output {
        if let Node::Tree(ref children) = self.node {
            &children[idx]
        } else {
            panic!("index called on leaf node");
        }
    }
}

impl<N, P, L> IndexMut<usize> for QuadTree<N, P, L> {
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
        let extent = Extent::from_positions(positions.iter().map(|(pos, _)| pos)).unwrap();
        let config = QuadTreeConfig { max_depth: 10 };
        QuadTree::<(), ()>::new(&extent, &config, positions.into_iter().collect());
    }
}
