use bevy::prelude::*;

use self::mass_moments::MassMoments;
use super::parameters::Parameters;
use super::LocalParticle;
use super::Timestep;
use crate::communication::Rank;
use crate::domain::quadtree;
use crate::domain::quadtree::Node;
use crate::domain::quadtree::QuadTreeConfig;
use crate::domain::GlobalExtent;
use crate::domain::Segment;
use crate::domain::Segments;
use crate::mass::Mass;
use crate::position::Position;
use crate::units;
use crate::units::Dimensionless;
use crate::units::Length;
use crate::units::VecAcceleration;
use crate::units::VecLength;
use crate::units::GRAVITY_CONSTANT;
use crate::velocity::Velocity;

mod mass_moments;

#[derive(Debug, Clone)]
pub struct ParticleData {
    pub entity: Entity,
    pub mass: units::Mass,
}

pub type LocalQuadTree = quadtree::QuadTree<MassMoments, ParticleData>;

#[derive(Debug)]
pub struct RemoteSegmentData {
    segment: Segment,
    rank: Rank,
    moments: MassMoments,
}

pub type RemoteQuadTree = quadtree::QuadTree<MassMoments, RemoteSegmentData>;

struct Solver {
    softening_length: Length,
    opening_angle: Dimensionless,
}

impl Solver {
    fn get_gravity_acceleration(
        &self,
        pos1: &VecLength,
        pos2: &VecLength,
        mass2: units::Mass,
    ) -> VecAcceleration {
        let distance_vector = *pos1 - *pos2;
        let distance = distance_vector.length() + self.softening_length;
        -distance_vector * GRAVITY_CONSTANT * mass2 / distance.cubed()
    }

    pub fn get_acceleration_on_particle(
        &self,
        tree: &LocalQuadTree,
        pos: VecLength,
        entity: Entity,
    ) -> VecAcceleration {
        match tree.node {
            Node::Tree(ref children) => children
                .iter()
                .map(|child| {
                    if self.opening_criterion(child, pos) {
                        self.get_gravity_acceleration(
                            &pos,
                            &child.data.center_of_mass(),
                            child.data.total(),
                        )
                    } else {
                        self.get_acceleration_on_particle(child, pos, entity)
                    }
                })
                .sum(),
            Node::Leaf(ref leaf) => leaf
                .iter()
                .filter(|(_, particle)| particle.entity != entity)
                .map(|(pos2, particle)| self.get_gravity_acceleration(&pos, pos2, particle.mass))
                .sum(),
        }
    }
    fn opening_criterion(&self, child: &LocalQuadTree, pos: VecLength) -> bool {
        let distance = pos.distance(&child.extents.center());
        let length = child.extents.max_side_length();
        length / distance < self.opening_angle
    }
}

pub(super) fn construct_local_quad_tree_system(
    mut commands: Commands,
    config: Res<QuadTreeConfig>,
    particles: Query<(Entity, &Position, &Mass)>,
    extent: Res<GlobalExtent>,
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
    let quadtree = LocalQuadTree::new(&extent.0, &config, particles);
    commands.insert_resource(quadtree);
}

pub(super) fn construct_remote_quad_tree_system(
    mut commands: Commands,
    config: Res<QuadTreeConfig>,
    segments: Res<Segments>,
    extent: Res<GlobalExtent>,
) {
    todo!()
    // let quadtree = RemoteQuadTree::new(&extent.0, &config, particles);
    // commands.insert_resource(quadtree);
}

pub(super) fn gravity_system(
    timestep: Res<Timestep>,
    tree: Option<Res<LocalQuadTree>>,
    mut particles: Query<(Entity, &Position, &mut Velocity), With<LocalParticle>>,
    parameters: Res<Parameters>,
) {
    if tree.is_none() {
        return;
    }
    let tree = tree.unwrap();
    let gravity = Solver {
        softening_length: parameters.softening_length,
        opening_angle: parameters.opening_angle,
    };
    for (entity, pos, mut vel) in particles.iter_mut() {
        let acceleration = gravity.get_acceleration_on_particle(&tree, pos.0, entity);
        vel.0 += acceleration * timestep.0;
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Entity;

    use super::LocalQuadTree;
    use super::ParticleData;
    use crate::domain::quadtree::QuadTreeConfig;
    use crate::domain::quadtree::{self};
    use crate::domain::Extent;
    use crate::physics::gravity::Solver;
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

    fn get_quadtree(n: i32) -> LocalQuadTree {
        let positions = get_positions(n);
        let extent = Extent::from_positions(positions.iter().map(|(pos, _)| pos)).unwrap();
        LocalQuadTree::new(&extent, &QuadTreeConfig::default(), positions)
    }

    #[test]
    fn mass_sum() {
        let quadtree = get_quadtree(7);
        check_all_sub_trees(&quadtree);
    }

    fn check_all_sub_trees(tree: &LocalQuadTree) {
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

    fn check_mass(tree: &LocalQuadTree) {
        let mut total = Mass::zero();
        tree.depth_first_map(&mut |_, data| total += data.iter().map(|(_, p)| p.mass).sum());
        assert_is_close(tree.data.total(), total);
    }

    #[test]
    fn compare_quadtree_gravity_to_direct_sum() {
        let n_particles = 50;
        let tree = get_quadtree(n_particles);
        let pos = Vec2Length::meter(3.5, 3.5);
        let solver = Solver {
            opening_angle: Dimensionless::zero(),
            softening_length: Length::zero(),
        };
        let acc1 = solver.get_acceleration_on_particle(&tree, pos, Entity::from_raw(0));
        let acc2 = direct_sum(&solver, &pos, get_positions(n_particles));
        let relative_diff = (acc1 - acc2).length() / (acc1.length() + acc2.length());
        // Precision is pretty low with f32, so change this to f64 once variable precision is implemented
        assert!(relative_diff.value() < &1e-5);
    }

    fn direct_sum(
        solver: &Solver,
        pos1: &Vec2Length,
        other_positions: Vec<(Vec2Length, ParticleData)>,
    ) -> Vec2Acceleration {
        let mut total = Vec2Acceleration::zero();
        for (pos2, data) in other_positions.iter() {
            total += solver.get_gravity_acceleration(pos1, pos2, data.mass);
        }
        total
    }
}
