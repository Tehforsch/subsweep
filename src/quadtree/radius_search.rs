use super::LeafDataType;
use super::Node;
use super::QuadTree;
use crate::config::TWO_TO_NUM_DIMENSIONS;
use crate::parameters::SimulationBox;
use crate::prelude::MVec;
use crate::units::Length;
use crate::units::VecLength;

#[cfg(feature = "3d")]
fn relative_bounding_box_overlap(dist: VecLength, total_size: VecLength) -> bool {
    (dist.x()).abs() <= total_size.x()
        && (dist.y()).abs() <= total_size.y()
        && (dist.z()).abs() <= total_size.z()
}

#[cfg(feature = "2d")]
fn relative_bounding_box_overlap(dist: VecLength, total_size: VecLength) -> bool {
    (dist.x()).abs() <= total_size.x() && (dist.y()).abs() <= total_size.y()
}

/// Returns whether the two bounding boxes given by
/// the center coordinates pos1 and pos2 and the side lengths
/// size1 and size2 overlap in a periodic box of size box_size
pub(super) fn bounding_boxes_overlap_periodic(
    box_: &SimulationBox,
    pos1: &VecLength,
    size1: &VecLength,
    pos2: &VecLength,
    size2: &VecLength,
) -> bool {
    let dist = box_.periodic_distance_vec(pos1, pos2);
    let total_size = *size1 + *size2;
    relative_bounding_box_overlap(dist, total_size)
}

fn within_radius_periodic(
    box_: &SimulationBox,
    pos1: &VecLength,
    pos2: &VecLength,
    radius: Length,
) -> bool {
    box_.periodic_distance(pos1, pos2) < radius
}

impl<N, L: LeafDataType> QuadTree<N, L> {
    pub fn iter_particles_in_radius<'a>(
        &'a self,
        box_size: &'a SimulationBox,
        pos: VecLength,
        radius: Length,
    ) -> impl Iterator<Item = &'a L> + 'a {
        let search = PeriodicRadiusSearch::new(box_size, pos, radius);
        TreeIter::new(self, search)
    }
}

impl<N, L> QuadTree<N, L> {
    pub fn iter<'a>(&'a self) -> TreeIter<'a, N, L, EntireTree> {
        TreeIter::new(self, EntireTree)
    }
}

struct StackItem<'a, N, L> {
    tree: &'a QuadTree<N, L>,
    pos_in_parent: usize,
    should_be_visited: bool,
}

impl<'a, N, L> Clone for StackItem<'a, N, L> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree.clone(),
            pos_in_parent: self.pos_in_parent.clone(),
            should_be_visited: self.should_be_visited,
        }
    }
}

pub struct TreeIter<'a, N, L, C> {
    stack: Vec<StackItem<'a, N, L>>,
    current_leaf_pos: usize,
    criterion: C,
}

impl<'a, N, L, C: SearchCriterion<N, L>> TreeIter<'a, N, L, C> {
    fn new(tree: &'a QuadTree<N, L>, criterion: C) -> Self {
        let mut iter = Self {
            criterion,
            stack: vec![],
            current_leaf_pos: 0,
        };
        let initial_stack_item = iter.get_stack_item_for_new_tree(tree, 0);
        iter.stack.push(initial_stack_item);
        iter
    }

    fn get_stack_item_for_new_tree(
        &self,
        tree: &'a QuadTree<N, L>,
        pos_in_parent: usize,
    ) -> StackItem<'a, N, L> {
        let should_be_visited = self.criterion.should_visit_node(tree);
        StackItem {
            pos_in_parent,
            tree,
            should_be_visited: should_be_visited,
        }
    }

    fn num_children(&self) -> usize {
        TWO_TO_NUM_DIMENSIONS
    }

    fn goto_next_node(&mut self) -> Option<()> {
        let last = self.stack.last()?.clone();
        if last.should_be_visited {
            match &last.tree.node {
                Node::Tree(tree) => {
                    // For the future: remember we visited this node
                    self.stack.last_mut().unwrap().should_be_visited = false;
                    // Then go deeper
                    self.stack
                        .push(self.get_stack_item_for_new_tree(&tree[0], 0));
                    return Some(());
                }
                Node::Leaf(_) => {}
            }
        }
        // If we encountered a leaf or a previously visited tree: Go to next child on this level, or up one level.
        let last = self.stack.pop().unwrap();
        let next_pos_in_parent = last.pos_in_parent + 1;
        let parent = self.stack.last()?;
        if next_pos_in_parent < self.num_children() {
            self.stack.push(self.get_stack_item_for_new_tree(
                &parent.tree.node.unwrap_tree()[next_pos_in_parent],
                next_pos_in_parent,
            ));
        }
        Some(())
    }

    fn get_current_node_if_it_should_be_visited(&self) -> Option<&'a Node<N, L>> {
        let last = self.stack.last()?;
        Some(&last.tree.node).filter(|_| last.should_be_visited)
    }
}

impl<'a, N, L, C: SearchCriterion<N, L>> Iterator for TreeIter<'a, N, L, C> {
    type Item = &'a L;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(Node::Leaf(leaf)) = self.get_current_node_if_it_should_be_visited() {
                let leaf = leaf.get(self.current_leaf_pos);
                if let Some(l) = leaf {
                    self.current_leaf_pos += 1;
                    if self.criterion.should_include_leaf(l) {
                        return Some(l);
                    } else {
                        continue;
                    }
                } else {
                    self.current_leaf_pos = 0;
                }
            }
            self.goto_next_node()?;
        }
    }
}

pub trait SearchCriterion<N, L> {
    fn should_visit_node(&self, tree: &QuadTree<N, L>) -> bool;
    fn should_include_leaf(&self, l: &L) -> bool;
}

pub struct EntireTree;

impl<N, L> SearchCriterion<N, L> for EntireTree {
    fn should_visit_node(&self, _: &QuadTree<N, L>) -> bool {
        true
    }

    fn should_include_leaf(&self, _: &L) -> bool {
        true
    }
}

#[derive(Debug)]
struct PeriodicRadiusSearch<'a> {
    box_size: &'a SimulationBox,
    pos: VecLength,
    radius: Length,
}

impl<'a> PeriodicRadiusSearch<'a> {
    fn new(box_size: &'a SimulationBox, pos: VecLength, radius: Length) -> Self {
        Self {
            box_size,
            pos,
            radius,
        }
    }
}

impl<'a, N, L: LeafDataType> SearchCriterion<N, L> for PeriodicRadiusSearch<'a> {
    fn should_visit_node(&self, tree: &QuadTree<N, L>) -> bool {
        bounding_boxes_overlap_periodic(
            self.box_size,
            &tree.extent.center(),
            &tree.extent.side_lengths(),
            &self.pos,
            &VecLength::from_vector_and_scale(MVec::ONE, self.radius),
        )
    }

    fn should_include_leaf(&self, particle: &L) -> bool {
        within_radius_periodic(self.box_size, &self.pos, particle.pos(), self.radius)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::domain::extent::Extent3d;
    use crate::domain::LeafData;
    use crate::parameters::SimulationBox;
    use crate::quadtree::Node;
    use crate::quadtree::QuadTree;
    use crate::quadtree::QuadTreeConfig;
    use crate::test_utils::get_particles;
    use crate::units::Length;
    use crate::units::VecLength;
    pub(super) fn direct_neighbour_search<'a>(
        particles: &'a [LeafData],
        pos: &VecLength,
        radius: &Length,
    ) -> Vec<&'a LeafData> {
        particles
            .iter()
            .filter(|particle| particle.pos.distance(pos) < *radius)
            .collect()
    }

    #[test]
    fn quadtree_iter() {
        let ex = || Extent3d::cube_from_side_length(Length::zero());
        let child1 = QuadTree {
            node: Node::Leaf(vec![1, 2, 3]),
            data: (),
            extent: ex(),
        };
        // let mut it = child1.iter();
        // assert_eq!(it.next(), Some(&1));
        // assert_eq!(it.next(), Some(&2));
        // assert_eq!(it.next(), Some(&3));
        // assert_eq!(it.next(), None);
        let child2 = QuadTree {
            node: Node::Leaf(vec![4, 5]),
            data: (),
            extent: ex(),
        };
        let child3 = QuadTree {
            node: Node::Leaf(vec![6]),
            data: (),
            extent: ex(),
        };
        let child4 = QuadTree {
            node: Node::Leaf(vec![7]),
            data: (),
            extent: ex(),
        };
        let child5 = QuadTree {
            node: Node::Leaf(vec![]),
            data: (),
            extent: ex(),
        };
        let child6 = QuadTree {
            node: Node::Leaf(vec![8]),
            data: (),
            extent: ex(),
        };
        let child7 = QuadTree {
            node: Node::Leaf(vec![]),
            data: (),
            extent: ex(),
        };
        let child8 = QuadTree {
            node: Node::Leaf(vec![9, 10]),
            data: (),
            extent: ex(),
        };
        let parent = QuadTree {
            node: Node::Tree(Box::new([
                child1, child2, child3, child4, child5, child6, child7, child8,
            ])),
            data: (),
            extent: ex(),
        };
        let mut it = parent.iter();
        assert_eq!(it.next(), Some(&1));
        assert_eq!(it.next(), Some(&2));
        assert_eq!(it.next(), Some(&3));
        assert_eq!(it.next(), Some(&4));
        assert_eq!(it.next(), Some(&5));
        assert_eq!(it.next(), Some(&6));
        assert_eq!(it.next(), Some(&7));
        assert_eq!(it.next(), Some(&8));
        assert_eq!(it.next(), Some(&9));
        assert_eq!(it.next(), Some(&10));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn radius_search() {
        let n = 12;
        let m = 12;
        let radius = Length::meters(2.0);
        let particles = get_particles(n, m);
        let particles: Vec<_> = particles
            .into_iter()
            .map(|particle| LeafData {
                id: particle.id,
                pos: particle.pos,
            })
            .collect();
        let extent = Extent3d::from_positions(particles.iter().map(|leaf| &leaf.pos)).unwrap();
        let tree: QuadTree<(), _> =
            QuadTree::new(&QuadTreeConfig::default(), particles.clone(), &extent);
        // We don't want this to periodically wrap, so make the simulation box large.
        let box_ = SimulationBox::new(Extent3d::cube_from_side_length(
            extent.side_lengths().x() * 10.0,
        ));
        for particle in particles.iter() {
            let tree_neighbours = tree.iter_particles_in_radius(&box_, particle.pos, radius);
            let direct_neighbours = direct_neighbour_search(&particles, &particle.pos, &radius);
            let tree_entities: HashSet<_> = tree_neighbours
                .into_iter()
                .map(|particle| particle.id)
                .collect();
            let direct_entities: HashSet<_> = direct_neighbours
                .into_iter()
                .map(|particle| particle.id)
                .collect();
            assert_eq!(tree_entities, direct_entities);
        }
    }
}
