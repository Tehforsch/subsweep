use bevy::prelude::*;

use super::parameters::HydrodynamicsParameters;
use super::HydroParticles;
use crate::components::Position;
use crate::components::SmoothingLength;
use crate::domain::GlobalExtent;
use crate::parameters::SimulationBox;
use crate::prelude::MVec;
use crate::quadtree::LeafDataType;
use crate::quadtree::NodeDataType;
use crate::quadtree::{self};
use crate::units::Length;
use crate::units::VecLength;

pub type QuadTree = quadtree::QuadTree<NodeData, LeafData>;

#[derive(Debug, Clone)]
pub struct LeafData {
    pub entity: Entity,
    pub pos: VecLength,
    pub smoothing_length: Length,
}

#[derive(Debug, Default)]
pub struct NodeData {
    pub largest_smoothing_length: Length,
}

impl LeafDataType for LeafData {
    fn pos(&self) -> &VecLength {
        &self.pos
    }
}

impl NodeDataType<LeafData> for NodeData {
    fn update_with(&mut self, leaf: &LeafData) {
        self.largest_smoothing_length = self.largest_smoothing_length.max(leaf.smoothing_length);
    }
}

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
/// size1 and size2. This function does not respect periodic
/// boundary conditions.
pub fn bounding_boxes_overlap_non_periodic(
    pos1: &VecLength,
    size1: &VecLength,
    pos2: &VecLength,
    size2: &VecLength,
) -> bool {
    let dist = *pos1 - *pos2;
    let total_size = *size1 + *size2;
    relative_bounding_box_overlap(dist, total_size)
}

fn add_particles_in_box<'a>(
    particles: &mut Vec<&'a LeafData>,
    tree: &'a QuadTree,
    box_size: &SimulationBox,
    pos: &VecLength,
    side_length: &Length,
) {
    let node_extent = tree.extent.side_lengths()
        + VecLength::from_vector_and_scale(MVec::ONE, tree.data.largest_smoothing_length);
    if bounding_boxes_overlap_periodic(
        box_size,
        &tree.extent.center(),
        &node_extent,
        pos,
        &VecLength::from_vector_and_scale(MVec::ONE, *side_length),
    ) {
        match &tree.node {
            quadtree::Node::Tree(tree) => {
                for child in tree.iter() {
                    add_particles_in_box(particles, child, box_size, pos, side_length);
                }
            }
            quadtree::Node::Leaf(leaf) => {
                particles.extend(leaf.iter());
            }
        }
    }
}

fn get_particles_in_box<'a>(
    tree: &'a QuadTree,
    box_size: &SimulationBox,
    pos: &VecLength,
    side_length: &Length,
) -> Vec<&'a LeafData> {
    let mut particles = vec![];
    add_particles_in_box(&mut particles, tree, box_size, pos, side_length);
    particles
}

fn particles_should_interact(
    box_: &SimulationBox,
    pos1: &VecLength,
    pos2: &VecLength,
    radius1: &Length,
    radius2: &Length,
) -> bool {
    box_.periodic_distance(pos1, pos2) < radius1.max(*radius2)
}

impl QuadTree {
    pub fn get_particles_in_radius<'a>(
        &'a self,
        box_size: &SimulationBox,
        pos: &VecLength,
        radius: &Length,
    ) -> Vec<&'a LeafData> {
        get_particles_in_box(self, box_size, pos, radius)
            .into_iter()
            .filter(|particle| {
                particles_should_interact(
                    box_size,
                    pos,
                    particle.pos(),
                    radius,
                    &particle.smoothing_length,
                )
            })
            .collect()
    }
}

pub(super) fn construct_quad_tree_system(
    parameters: Res<HydrodynamicsParameters>,
    particles: HydroParticles<(Entity, &Position, &SmoothingLength)>,
    extent: Res<GlobalExtent>,
    mut quadtree: ResMut<QuadTree>,
) {
    let particles: Vec<_> = particles
        .iter()
        .map(|(entity, pos, smoothing_length)| LeafData {
            entity,
            pos: pos.0,
            smoothing_length: **smoothing_length,
        })
        .collect();
    *quadtree = QuadTree::new(&parameters.tree, particles, &extent);
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::particles_should_interact;
    use super::LeafData;
    use super::QuadTree;
    use crate::domain::extent::Extent;
    use crate::parameters::SimulationBox;
    use crate::quadtree::QuadTreeConfig;
    use crate::test_utils::get_particles;
    use crate::units::Length;
    use crate::units::VecLength;

    pub(super) fn direct_neighbour_search<'a>(
        particles: &'a [LeafData],
        box_size: &SimulationBox,
        pos: &VecLength,
        radius: &Length,
    ) -> Vec<&'a LeafData> {
        particles
            .iter()
            .filter(|particle| {
                particles_should_interact(
                    box_size,
                    pos,
                    &particle.pos,
                    radius,
                    &particle.smoothing_length,
                )
            })
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
                entity: particle.entity,
                pos: particle.pos,
                smoothing_length: particle.pos.x() * 0.2,
            })
            .collect();
        let extent = Extent::from_positions(particles.iter().map(|leaf| &leaf.pos)).unwrap();
        let box_size = extent.clone().into();
        let tree = QuadTree::new(&QuadTreeConfig::default(), particles.clone(), &extent);
        let entities_as_hash_set = |leaf_data_vec: Vec<&LeafData>| {
            leaf_data_vec
                .into_iter()
                .map(|particle| particle.entity)
                .collect::<HashSet<_>>()
        };
        for particle in particles.iter() {
            let tree_neighbours = tree.get_particles_in_radius(&box_size, &particle.pos, &radius);
            let direct_neighbours =
                direct_neighbour_search(&particles, &box_size, &particle.pos, &radius);
            assert_eq!(
                entities_as_hash_set(tree_neighbours),
                entities_as_hash_set(direct_neighbours)
            );
        }
    }

    #[test]
    #[rustfmt::skip]
    #[cfg(not(feature = "2d"))]
    fn bounding_boxes_overlap_periodic() {
        let test = |box_size, (x1, y1, z1), (lx1, ly1, lz1), (x2, y2, z2), (lx2, ly2, lz2), v| {
            assert_eq!(
                super::bounding_boxes_overlap_periodic(
                    box_size,
                    &VecLength::meters(x1, y1, z1),
                    &VecLength::meters(lx1, ly1, lz1),
                    &VecLength::meters(x2, y2, z2),
                    &VecLength::meters(lx2, ly2, lz2)
                ),
                v
            );
        };
        let box_size = SimulationBox::cube_from_side_length(Length::meters(1.0));
        test(&box_size, (0.1, 0.1, 0.1), (0.05, 0.05, 0.05), (0.2, 0.2, 0.2), (0.0, 0.0, 0.0), false);
        test(&box_size, (0.1, 0.1, 0.1), (0.15, 0.15, 0.15), (0.2, 0.2, 0.2), (0.0, 0.0, 0.0), true);
        test(&box_size, (0.1, 0.1, 0.1), (0.1, 0.1, 0.1), (0.9, 0.9, 0.9), (0.2, 0.2, 0.2), true);
        test(&box_size, (-0.1, -0.1, -0.1), (0.001, 0.001, 0.001), (0.9, 0.9, 0.9), (0.0, 0.0, 0.0), true);
    }
}
