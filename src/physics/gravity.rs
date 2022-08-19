use bevy::prelude::*;

use super::LocalParticle;
use super::Timestep;
use crate::domain::quadtree::Node;
use crate::domain::quadtree::QuadTree;
use crate::mass::Mass;
use crate::position::Position;
use crate::units::vec2::Acceleration;
use crate::units::vec2::Length;
use crate::units::GRAVITY_CONSTANT;
use crate::velocity::Velocity;

fn get_gravity_acceleration(
    pos1: Length,
    pos2: Length,
    mass2: crate::units::f32::Mass,
) -> Acceleration {
    let distance_vector = pos1 - pos2;
    let distance = distance_vector.length();
    -distance_vector * GRAVITY_CONSTANT * mass2 / distance.cubed()
}

pub fn get_acceleration_on_particle(tree: &QuadTree, pos: Length, entity: Entity) -> Acceleration {
    match tree.data {
        Node::Node(ref children) => children
            .iter()
            .map(|child| get_acceleration_on_particle(child, pos, entity))
            .sum(),
        Node::Leaf(ref leaf) => leaf
            .particles
            .iter()
            .filter(|particle| particle.entity != entity)
            .map(|particle| get_gravity_acceleration(pos, particle.pos, particle.mass))
            .sum(),
    }
}

pub(super) fn construct_quad_tree_system(
    mut commands: Commands,
    particles: Query<(Entity, &Position, &Mass)>,
) {
    let particles: Vec<_> = particles
        .iter()
        .map(|(entity, pos, mass)| (pos.0, mass.0, entity))
        .collect();
    let quadtree = QuadTree::new(particles);
    commands.insert_resource(quadtree);
}

pub(super) fn gravity_system(
    timestep: Res<Timestep>,
    tree: Res<QuadTree>,
    mut particles: Query<(Entity, &Position, &mut Velocity), With<LocalParticle>>,
) {
    for (entity, pos, mut vel) in particles.iter_mut() {
        let acceleration = get_acceleration_on_particle(&tree, pos.0, entity);
        vel.0 += acceleration * timestep.0;
    }
}
