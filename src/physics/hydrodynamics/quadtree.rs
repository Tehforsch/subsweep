use bevy::prelude::*;

use super::parameters::HydrodynamicsParameters;
use crate::domain::GlobalExtent;
use crate::physics::LocalParticle;
use crate::prelude::Position;
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

fn bounding_boxes_overlap(
    pos1: &VecLength,
    size1: &VecLength,
    pos2: &VecLength,
    size2: &VecLength,
) -> bool {
    (pos1.x() - pos2.x()).abs() < size1.x() + size2.x()
        && (pos1.y() - pos2.y()).abs() < size1.y() + size2.y()
}

fn add_particles_in_box<'a>(
    particles: &mut Vec<&'a LeafData>,
    tree: &'a QuadTree,
    pos: &VecLength,
    side_length: &Length,
) {
    let node_extent = tree.extent.side_lengths()
        + VecLength::new(
            tree.data.largest_smoothing_length,
            tree.data.largest_smoothing_length,
        );
    if bounding_boxes_overlap(
        &tree.extent.center(),
        &node_extent,
        pos,
        &VecLength::new(*side_length, *side_length),
    ) {
        match &tree.node {
            quadtree::Node::Tree(tree) => {
                for child in tree.iter() {
                    add_particles_in_box(particles, child, pos, side_length);
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
    pos: &VecLength,
    side_length: &Length,
) -> Vec<&'a LeafData> {
    let mut particles = vec![];
    add_particles_in_box(&mut particles, tree, pos, side_length);
    particles
}

pub(super) fn get_particles_in_radius<'a>(
    tree: &'a QuadTree,
    pos: &VecLength,
    radius: &Length,
) -> Vec<&'a LeafData> {
    get_particles_in_box(tree, pos, radius)
        .into_iter()
        .filter(|particle| particle.pos().distance(pos) < *radius)
        .collect()
}

pub(super) fn construct_quad_tree_system(
    parameters: Res<HydrodynamicsParameters>,
    particles: Query<(Entity, &Position), With<LocalParticle>>,
    extent: Res<GlobalExtent>,
    mut quadtree: ResMut<QuadTree>,
) {
    let particles: Vec<_> = particles
        .iter()
        .map(|(entity, pos)| LeafData {
            entity,
            pos: pos.0,
            smoothing_length: parameters.smoothing_length,
        })
        .collect();
    *quadtree = QuadTree::new(&parameters.tree, particles, &extent);
}
