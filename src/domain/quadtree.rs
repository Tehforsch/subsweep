use std::ops::Index;
use std::ops::IndexMut;

use bevy::prelude::Entity;

use super::Extents;
use crate::position::Position;
use crate::units::vec2;

#[derive(Debug, Default)]
struct LeafData {
    particles: Vec<(vec2::Length, Entity)>,
}

#[derive(Debug)]
enum Node {
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
struct QuadTree {
    data: Node,
    extents: Extents,
}

impl QuadTree {
    pub fn new<'a>(particles: &[(vec2::Length, Entity)]) -> Self {
        let extents = Extents::from_positions(particles.iter().map(|(pos, _)| pos))
            .expect("Not enough particles to construct quadtree");
        let mut tree = Self::make_empty_leaf_from_extents(extents);
        for (pos, entity) in particles.iter() {
            tree.insert(pos.clone(), entity.clone());
        }
        tree
    }

    fn insert(&mut self, pos: vec2::Length, entity: Entity) {
        if let Node::Leaf(ref mut leaf) = self.data {
            if leaf.particles.is_empty() {
                leaf.particles.push((pos, entity));
                return;
            } else {
                self.subdivide();
            }
        }
        if let Node::Node(ref mut children) = self.data {
            let quadrant = &mut children[self.extents.get_quadrant_index(&pos)];
            quadrant.insert(pos, entity);
        }
    }

    fn subdivide(&mut self) {
        debug_assert!(matches!(self.data, Node::Leaf(_)));
        let quadrants = self.extents.get_quadrants();
        let children = Box::new(quadrants.map(Self::make_empty_leaf_from_extents));
        let particles = self.data.make_node(children);
        for (pos, entity) in particles.particles.into_iter() {
            self.insert(pos, entity);
        }
    }

    fn make_empty_leaf_from_extents(extents: Extents) -> Self {
        Self {
            data: Node::Leaf(LeafData::default()),
            extents,
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
    use crate::domain::extents::Extents;
    use crate::units::f32::meter;

    #[test]
    fn construct_tree() {
        let positions = &[
            (vec2::meter(Vec2::new(1.0, 1.0)), Entity::from_raw(0)),
            (vec2::meter(Vec2::new(-1.0, 1.0)), Entity::from_raw(1)),
            (vec2::meter(Vec2::new(1.0, -1.0)), Entity::from_raw(2)),
            (vec2::meter(Vec2::new(-1.0, -1.0)), Entity::from_raw(3)),
        ];
        let tree = QuadTree::new(positions);
        dbg!(tree);
        assert!(false);
    }
}
