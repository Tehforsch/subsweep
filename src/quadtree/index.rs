use std::ops::Index;
use std::ops::IndexMut;

use mpi::traits::Equivalence;

use super::node_index::NodeIndex;
use super::Node;
use super::QuadTree;
use super::MAX_DEPTH;
use crate::config::TWO_TO_NUM_DIMENSIONS;

#[derive(Equivalence, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct QuadTreeIndex([u8; MAX_DEPTH]);

impl std::fmt::Debug for QuadTreeIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self
            .0
            .map(|x| <u8 as Into<NodeIndex>>::into(x).to_string())
            .join("");
        write!(f, "QuadTreeIndex({s})")
    }
}

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
            Box::new((0..TWO_TO_NUM_DIMENSIONS).flat_map(move |num_child| {
                current_index.0[current_depth] = NodeIndex::Child(num_child as u8).into();
                Self::internal_iter_all_at_depth(depth, current_index, current_depth + 1)
            }))
        } else {
            let mut current_index = current_index;
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

impl<N, L> Index<&QuadTreeIndex> for QuadTree<N, L> {
    type Output = QuadTree<N, L>;

    fn index(&self, idx: &QuadTreeIndex) -> &Self::Output {
        self.index_into_depth(idx, 0)
    }
}

impl<N, L> IndexMut<&QuadTreeIndex> for QuadTree<N, L> {
    fn index_mut(&mut self, index: &QuadTreeIndex) -> &mut Self::Output {
        self.index_into_depth_mut(index, 0)
    }
}

impl<N, L> QuadTree<N, L> {
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

    fn index_into_depth_mut(&mut self, idx: &QuadTreeIndex, depth: usize) -> &mut Self {
        match idx.0[depth].into() {
            NodeIndex::ThisNode => self,
            NodeIndex::Child(num) => {
                if let Node::Tree(ref mut children) = self.node {
                    children[num as usize].index_into_depth_mut(idx, depth + 1)
                } else {
                    panic!("Invalid index");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Entity;

    use super::super::node_index::NodeIndex;
    use super::QuadTreeIndex;
    use crate::domain::extent::Extent;
    use crate::domain::LeafData;
    use crate::quadtree::tests::get_min_depth_quadtree;
    use crate::quadtree::Node;
    use crate::quadtree::QuadTree;
    use crate::quadtree::QuadTreeConfig;

    #[test]
    fn quadtree_index() {
        let min_depth = 5;
        let mut tree: QuadTree<(), LeafData> = get_min_depth_quadtree(min_depth);
        // obtain a list of particles we can add into the quadtree
        // from the centers of all the leaf ectents
        let config = QuadTreeConfig::default();
        let mut particles = vec![];
        tree.depth_first_map_leaf(&mut |extent: &Extent, _| {
            particles.push(extent.center());
        });
        for pos in particles.into_iter() {
            let data = LeafData {
                pos,
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
        assert!(index1.belongs_to(&index1));
        assert!(index1.belongs_to(&index2));
        assert!(!index2.belongs_to(&index1));
        assert!(index2.belongs_to(&index2));
        assert!(!index1.belongs_to(&index3));
        assert!(!index3.belongs_to(&index1));
    }
}
