use bevy::prelude::*;

use super::parameters::Parameters;
use super::LocalParticle;
use super::Timestep;
use crate::domain::quadtree;
use crate::domain::quadtree::Node;
use crate::domain::quadtree::QuadTreeConfig;
use crate::mass::Mass;
use crate::position::Position;
use crate::units;
use crate::units::Length;
use crate::units::VecAcceleration;
use crate::units::VecLength;
use crate::units::GRAVITY_CONSTANT;
use crate::velocity::Velocity;

#[derive(Debug, Clone)]
pub struct ParticleData {
    pub entity: Entity,
    pub mass: units::Mass,
}

pub type QuadTree = quadtree::QuadTree<(), ParticleData>;

fn get_gravity_acceleration(
    pos1: &VecLength,
    pos2: &VecLength,
    mass2: units::Mass,
    softening_length: Length,
) -> VecAcceleration {
    let distance_vector = *pos1 - *pos2;
    let distance = distance_vector.length() + softening_length;
    -distance_vector * GRAVITY_CONSTANT * mass2 / distance.cubed()
}

pub fn get_acceleration_on_particle(
    tree: &QuadTree,
    pos: VecLength,
    entity: Entity,
    softening_length: Length,
) -> VecAcceleration {
    match tree.node {
        Node::Node(ref children) => children
            .iter()
            .map(|child| get_acceleration_on_particle(child, pos, entity, softening_length))
            .sum(),
        Node::Leaf(ref leaf) => leaf
            .iter()
            .filter(|(_, particle)| particle.entity != entity)
            .map(|(pos2, particle)| {
                get_gravity_acceleration(&pos, pos2, particle.mass, softening_length)
            })
            .sum(),
    }
}

pub(super) fn construct_quad_tree_system(
    mut commands: Commands,
    config: Res<QuadTreeConfig>,
    particles: Query<(Entity, &Position, &Mass)>,
) {
    let particles: Vec<_> = particles
        .iter()
        .map(|(entity, pos, mass)| {
            (
                pos.0,
                ParticleData {
                    mass: mass.0,
                    entity,
                },
            )
        })
        .collect();
    let quadtree = QuadTree::new(&config, particles);
    commands.insert_resource(quadtree);
}

pub(super) fn gravity_system(
    timestep: Res<Timestep>,
    tree: Res<QuadTree>,
    mut particles: Query<(Entity, &Position, &mut Velocity), With<LocalParticle>>,
    parameters: Res<Parameters>,
) {
    for (entity, pos, mut vel) in particles.iter_mut() {
        let acceleration =
            get_acceleration_on_particle(&tree, pos.0, entity, parameters.softening_length);
        vel.0 += acceleration * timestep.0;
    }
}
