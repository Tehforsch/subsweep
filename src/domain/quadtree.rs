use std::ops::Index;
use std::ops::IndexMut;

use bevy::prelude::Entity;

use super::Extents;
use crate::units::f32::Mass;
use crate::units::vec2;

#[derive(Debug)]
pub struct ParticleData {
    pub entity: Entity,
    pub pos: vec2::Length,
    pub mass: Mass,
}

#[derive(Debug, Default)]
pub struct LeafData {
    pub particles: Vec<ParticleData>,
}

#[derive(Debug)]
pub enum Node {
    Node(Box<[QuadTree; 4]>),
    Leaf(LeafData),
}

impl Node {
    fn make_node(&mut self, children: Box<[QuadTree; 4]>) -> LeafData {
        let value = std::mem::replace(self, Node::Node(children));
        if let Self::Leaf(leaf) = value {
            leaf
        } else {
            panic!("make_node called on Node value")
        }
    }
}

#[derive(Debug)]
pub struct QuadTree {
    pub data: Node,
    pub extents: Extents,
}

impl QuadTree {
    pub fn new<'a>(particles: Vec<(vec2::Length, Mass, Entity)>) -> Self {
        let extents = Extents::from_positions(particles.iter().map(|particle| &particle.0))
            .expect("Not enough particles to construct quadtree");
        let mut tree = Self::make_empty_leaf_from_extents(extents);
        for (pos, mass, entity) in particles.iter() {
            tree.insert(ParticleData {
                pos: pos.clone(),
                entity: entity.clone(),
                mass: mass.clone(),
            });
        }
        tree
    }

    fn insert(&mut self, particle: ParticleData) {
        if let Node::Leaf(ref mut leaf) = self.data {
            if leaf.particles.is_empty() {
                leaf.particles.push(particle);
                return;
            } else {
                self.subdivide();
            }
        }
        if let Node::Node(ref mut children) = self.data {
            let quadrant = &mut children[self.extents.get_quadrant_index(&particle.pos)];
            quadrant.insert(particle);
        }
    }

    fn subdivide(&mut self) {
        debug_assert!(matches!(self.data, Node::Leaf(_)));
        let quadrants = self.extents.get_quadrants();
        let children = Box::new(quadrants.map(Self::make_empty_leaf_from_extents));
        let particles = self.data.make_node(children);
        for particle in particles.particles.into_iter() {
            self.insert(particle);
        }
    }

    fn make_empty_leaf_from_extents(extents: Extents) -> Self {
        Self {
            data: Node::Leaf(LeafData::default()),
            extents,
        }
    }

    pub fn depth_first_map(&self, closure: &mut impl FnMut(&Extents) -> ()) {
        match self.data {
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

impl Index<usize> for QuadTree {
    type Output = QuadTree;

    fn index(&self, idx: usize) -> &Self::Output {
        if let Node::Node(ref children) = self.data {
            &children[idx]
        } else {
            panic!("index called on leaf node");
        }
    }
}

impl IndexMut<usize> for QuadTree {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        if let Node::Node(ref mut children) = self.data {
            &mut children[idx]
        } else {
            panic!("index_mut called on leaf node");
        }
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;
    use crate::units::f32::kilogram;

    #[test]
    fn no_infinite_recursion_in_tree_construction_with_close_particles() {
        assert!(false);
        let positions = [
            (
                vec2::meter(Vec2::new(1.0, 1.0)),
                kilogram(1.0),
                Entity::from_raw(0),
            ),
            (
                vec2::meter(Vec2::new(1.0, 1.0)),
                kilogram(1.0),
                Entity::from_raw(0),
            ),
        ];
        let tree = QuadTree::new(positions.into_iter().collect());
        dbg!(&tree);
    }
}
