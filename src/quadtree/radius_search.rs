use super::LeafDataType;
use super::QuadTree;
use crate::parameters::SimulationBox;
use crate::prelude::MVec;
use crate::quadtree::{self};
use crate::units::Length;
use crate::units::VecLength;

fn relative_bounding_box_overlap(dist: VecLength, total_size: VecLength) -> bool {
    (dist.x()).abs() < total_size.x() && (dist.y()).abs() < total_size.y()
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

/// Returns whether the two bounding boxes given by
/// the center coordinates pos1 and pos2 and the side lengths
/// size1 and size2 overlap in a periodic box of size box_size
pub fn bounding_boxes_overlap(
    _box_: &SimulationBox,
    pos1: &VecLength,
    size1: &VecLength,
    pos2: &VecLength,
    size2: &VecLength,
) -> bool {
    let dist = *pos1 - *pos2;
    let total_size = *size1 + *size2;
    relative_bounding_box_overlap(dist, total_size)
}

fn within_radius(
    _box_: &SimulationBox,
    pos1: &VecLength,
    pos2: &VecLength,
    radius: Length,
) -> bool {
    pos1.distance(pos2) < radius
    // box_.periodic_distance(pos1, pos2) < *radius
}

impl<N, L: LeafDataType> QuadTree<N, L> {
    fn add_particles_for_criterion<'a, C: SearchCriterion<N, L>>(
        &'a self,
        particles: &mut Vec<&'a L>,
        criterion: &C,
    ) {
        if criterion.should_check_node(self) {
            match &self.node {
                quadtree::Node::Tree(tree) => {
                    for child in tree.iter() {
                        child.add_particles_for_criterion(particles, criterion);
                    }
                }
                quadtree::Node::Leaf(leaf) => {
                    particles.extend(leaf.iter());
                }
            }
        }
    }

    fn get_particles_by_criterion<'a, C: SearchCriterion<N, L>>(
        &'a self,
        criterion: &C,
    ) -> Vec<&'a L> {
        let mut particles = vec![];
        self.add_particles_for_criterion(&mut particles, criterion);
        particles
            .into_iter()
            .filter(|p| criterion.particle_included(p))
            .collect()
    }

    pub fn get_particles_in_radius<'a>(
        &'a self,
        box_size: &SimulationBox,
        pos: &VecLength,
        radius: &Length,
    ) -> Vec<&'a L> {
        self.get_particles_by_criterion(&RadiusSearch::new(box_size, pos, radius))
    }
}

trait SearchCriterion<N, L> {
    fn should_check_node(&self, tree: &QuadTree<N, L>) -> bool;
    fn particle_included(&self, l: &L) -> bool;
}

struct RadiusSearch<'a> {
    box_size: &'a SimulationBox,
    pos: VecLength,
    radius: Length,
}

impl<'a> RadiusSearch<'a> {
    fn new(box_size: &'a SimulationBox, pos: &VecLength, radius: &Length) -> Self {
        Self {
            box_size,
            pos: *pos,
            radius: *radius,
        }
    }
}

impl<'a, N, L: LeafDataType> SearchCriterion<N, L> for RadiusSearch<'a> {
    fn should_check_node(&self, tree: &QuadTree<N, L>) -> bool {
        bounding_boxes_overlap(
            self.box_size,
            &tree.extent.center(),
            &tree.extent.side_lengths(),
            &self.pos,
            &VecLength::from_vector_and_scale(MVec::ONE, self.radius),
        )
    }

    fn particle_included(&self, particle: &L) -> bool {
        within_radius(self.box_size, &self.pos, particle.pos(), self.radius)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::domain::extent::Extent3d;
    use crate::domain::LeafData;
    use crate::parameters::SimulationBox;
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
        let box_ = SimulationBox::new(extent);
        for particle in particles.iter() {
            let tree_neighbours = tree.get_particles_in_radius(&box_, &particle.pos, &radius);
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
