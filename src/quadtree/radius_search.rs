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

fn within_radius(
    box_: &SimulationBox,
    pos1: &VecLength,
    pos2: &VecLength,
    radius: &Length,
) -> bool {
    box_.periodic_distance(pos1, pos2) < *radius
}

impl<N, L: LeafDataType> QuadTree<N, L> {
    fn add_particles_in_box<'a>(
        &'a self,
        particles: &mut Vec<&'a L>,
        box_size: &SimulationBox,
        pos: &VecLength,
        side_length: &Length,
    ) {
        let node_extent = self.extent.side_lengths();
        if bounding_boxes_overlap_periodic(
            box_size,
            &self.extent.center(),
            &node_extent,
            pos,
            &VecLength::from_vector_and_scale(MVec::ONE, *side_length),
        ) {
            match &self.node {
                quadtree::Node::Tree(tree) => {
                    for child in tree.iter() {
                        child.add_particles_in_box(particles, box_size, pos, side_length);
                    }
                }
                quadtree::Node::Leaf(leaf) => {
                    particles.extend(leaf.iter());
                }
            }
        }
    }

    fn get_particles_in_box<'a>(
        &'a self,
        box_size: &SimulationBox,
        pos: &VecLength,
        side_length: &Length,
    ) -> Vec<&'a L> {
        let mut particles = vec![];
        self.add_particles_in_box(&mut particles, box_size, pos, side_length);
        particles
    }

    pub fn get_particles_in_radius<'a>(
        &'a self,
        box_size: &SimulationBox,
        pos: &VecLength,
        radius: &Length,
    ) -> Vec<&'a L> {
        self.get_particles_in_box(box_size, pos, radius)
            .into_iter()
            .filter(|particle| within_radius(box_size, pos, particle.pos(), radius))
            .collect()
    }
}
