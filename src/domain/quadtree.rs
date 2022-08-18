use std::ops::Index;
use std::ops::IndexMut;

use super::Extents;

enum Node {
    Node(Box<[QuadTree; 4]>),
    Leaf(LeafData),
}

struct QuadTree {
    data: Node,
    extents: Extents,
}

impl QuadTree {
    fn subdivide(&mut self) {
        debug_assert!(matches!(self.data, Node::Leaf(_)));
        let quadrants = self.extents.get_quadrants();
        let make_leaf = |extents| QuadTree {
            data: Node::Leaf(LeafData),
            extents,
        };
        let children = Box::new(quadrants.map(make_leaf));
        self.data = Node::Node(children);
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

struct LeafData;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::extents::Extents;
    use crate::units::f32::meter;

    #[test]
    fn construct_tree() {
        let root_extents = Extents::new(meter(0.0), meter(4.0), meter(0.0), meter(8.0));
        let mut tree = QuadTree {
            data: Node::Leaf(LeafData),
            extents: root_extents,
        };
    }
}
