use bevy::prelude::*;

use super::parameters::Parameters;
use super::LocalParticle;
use super::Timestep;
use crate::domain::quadtree;
use crate::domain::quadtree::Node;
use crate::domain::quadtree::NodeDataType;
use crate::domain::quadtree::QuadTreeConfig;
use crate::mass::Mass;
use crate::position::Position;
use crate::units;
use crate::units::Dimensionless;
use crate::units::Length;
use crate::units::VecAcceleration;
use crate::units::VecLength;
use crate::units::VecLengthMass;
use crate::units::GRAVITY_CONSTANT;
use crate::velocity::Velocity;

#[derive(Debug, Clone)]
pub struct ParticleData {
    pub entity: Entity,
    pub mass: units::Mass,
}

#[derive(Default, Debug)]
pub struct MassMoments {
    total: units::Mass,
    weighted_position_sum: VecLengthMass,
    count: usize,
}

impl MassMoments {
    fn center_of_mass(&self) -> VecLength {
        if self.count == 0 {
            return VecLength::zero();
        }
        self.weighted_position_sum / self.total
    }
}
impl NodeDataType<ParticleData> for MassMoments {
    fn add_new_leaf_data(&mut self, pos: &VecLength, data: &ParticleData) {
        self.count += 1;
        self.total += data.mass;
        self.weighted_position_sum += *pos * data.mass;
    }
}

pub type QuadTree = quadtree::QuadTree<MassMoments, ParticleData>;

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
    opening_angle: Dimensionless,
) -> VecAcceleration {
    match tree.node {
        Node::Tree(ref children) => children
            .iter()
            .map(|child| {
                if opening_criterion(child, pos, opening_angle) {
                    get_gravity_acceleration(
                        &pos,
                        &child.data.center_of_mass(),
                        child.data.total,
                        softening_length,
                    )
                } else {
                    get_acceleration_on_particle(
                        child,
                        pos,
                        entity,
                        softening_length,
                        opening_angle,
                    )
                }
            })
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

fn opening_criterion(child: &QuadTree, pos: VecLength, opening_angle: Dimensionless) -> bool {
    let distance = pos.distance(&child.extents.center());
    let length = child.extents.max_side_length();
    length / distance < opening_angle
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
        let acceleration = get_acceleration_on_particle(
            &tree,
            pos.0,
            entity,
            parameters.softening_length,
            parameters.opening_angle,
        );
        vel.0 += acceleration * timestep.0;
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Entity;

    use super::get_acceleration_on_particle;
    use super::get_gravity_acceleration;
    use super::ParticleData;
    use super::QuadTree;
    use crate::domain::quadtree::QuadTreeConfig;
    use crate::domain::quadtree::{self};
    use crate::units::assert_is_close;
    use crate::units::Dimensionless;
    use crate::units::Length;
    use crate::units::Mass;
    use crate::units::Vec2Acceleration;
    use crate::units::Vec2Length;

    fn get_positions(n: i32) -> Vec<(Vec2Length, ParticleData)> {
        (1..n)
            .flat_map(move |x| {
                (1..n).map(move |y| {
                    (
                        Vec2Length::meter(x as f32, y as f32),
                        ParticleData {
                            mass: Mass::kilogram(x as f32 * y as f32),
                            entity: Entity::from_raw(n as u32),
                        },
                    )
                })
            })
            .collect()
    }

    #[test]
    fn mass_sum() {
        let quadtree = QuadTree::new(&QuadTreeConfig::default(), get_positions(7));
        check_all_sub_trees(&quadtree);
    }

    fn check_all_sub_trees(tree: &QuadTree) {
        check_mass(tree);
        match tree.node {
            quadtree::Node::Tree(ref children) => {
                for child in children.iter() {
                    check_all_sub_trees(child)
                }
            }
            quadtree::Node::Leaf(_) => {}
        }
    }

    fn check_mass(tree: &QuadTree) {
        let mut total = Mass::zero();
        tree.depth_first_map(&mut |_, data| total += data.iter().map(|(_, p)| p.mass).sum());
        assert_is_close(tree.data.total, total);
    }

    #[test]
    fn compare_quadtree_gravity_to_direct_sum() {
        let n_particles = 50;
        let tree = QuadTree::new(&QuadTreeConfig::default(), get_positions(n_particles));
        let pos = Vec2Length::meter(3.5, 3.5);
        let acc1 = get_acceleration_on_particle(
            &tree,
            pos,
            Entity::from_raw(0),
            Length::zero(),
            Dimensionless::zero(),
        );
        let acc2 = direct_sum(&pos, get_positions(n_particles));
        let relative_diff = (acc1 - acc2).length() / (acc1.length() + acc2.length());
        // Precision is pretty low with f32, so change this to f64 once variable precision is implemented
        assert!(relative_diff.value() < &1e-5);
    }

    fn direct_sum(
        pos1: &Vec2Length,
        other_positions: Vec<(Vec2Length, ParticleData)>,
    ) -> Vec2Acceleration {
        let mut total = Vec2Acceleration::zero();
        for (pos2, data) in other_positions.iter() {
            total += get_gravity_acceleration(pos1, pos2, data.mass, Length::zero());
        }
        total
    }
}
