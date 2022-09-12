use bevy::prelude::*;

use super::parameters::Parameters;
use super::LocalParticle;
use super::Timestep;
use crate::domain::quadtree::Node;
use crate::domain::quadtree::QuadTree;
use crate::position::Position;
use crate::units;
use crate::units::Dimensionless;
use crate::units::Length;
use crate::units::VecAcceleration;
use crate::units::VecLength;
use crate::units::GRAVITY_CONSTANT;
use crate::velocity::Velocity;

pub(super) mod mass_moments;

pub(super) mod plugin;

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
        tree: &QuadTree,
        pos: VecLength,
        entity: Entity,
    ) -> VecAcceleration {
        match tree.node {
            Node::Tree(ref children) => children
                .iter()
                .map(|child| {
                    if self.should_be_opened(child, pos) {
                        self.get_acceleration_on_particle(child, pos, entity)
                    } else {
                        self.get_gravity_acceleration(
                            &pos,
                            &child.data.moments.center_of_mass(),
                            child.data.moments.total(),
                        )
                    }
                })
                .sum(),
            Node::Leaf(ref leaf) => leaf
                .iter()
                .map(|particle| self.get_gravity_acceleration(&pos, &particle.pos, particle.mass))
                .sum(),
        }
    }

    fn should_be_opened(&self, child: &QuadTree, pos: VecLength) -> bool {
        let distance = pos.distance(&child.extent.center());
        let length = child.extent.max_side_length();
        length / distance > self.opening_angle
    }
}

pub(super) fn gravity_system(
    timestep: Res<Timestep>,
    tree: Option<Res<QuadTree>>,
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
        let acceleration = gravity.get_acceleration_on_particle(&tree, **pos, entity);
        **vel += acceleration * **timestep;
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Entity;

    use super::QuadTree;
    use crate::domain::extent::Extent;
    use crate::domain::quadtree::LeafData;
    use crate::domain::quadtree::QuadTreeConfig;
    use crate::domain::quadtree::{self};
    use crate::physics::gravity::Solver;
    use crate::units::assert_is_close;
    use crate::units::Dimensionless;
    use crate::units::Length;
    use crate::units::Mass;
    use crate::units::Vec2Acceleration;
    use crate::units::Vec2Length;

    fn get_particles(n: i32) -> Vec<LeafData> {
        (1..n)
            .flat_map(move |x| {
                (1..n).map(move |y| LeafData {
                    entity: Entity::from_raw((x * n + y) as u32),
                    pos: Vec2Length::meter(x as f32, y as f32),
                    mass: Mass::kilogram(x as f32 * y as f32),
                })
            })
            .collect()
    }

    fn get_tree_for_particles(n: i32) -> QuadTree {
        let particles = get_particles(n);
        let extent = Extent::from_positions(particles.iter().map(|part| &part.pos)).unwrap();
        QuadTree::new(&QuadTreeConfig::default(), particles, &extent)
    }

    #[test]
    fn mass_sum() {
        let tree = get_tree_for_particles(7);
        check_all_sub_trees(&tree);
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
        tree.depth_first_map_leaf(&mut |_, data| total += data.iter().map(|p| p.mass).sum());
        assert_is_close(tree.data.moments.total(), total);
    }

    #[test]
    fn compare_quadtree_gravity_to_direct_sum() {
        let n_particles = 50;
        let tree = get_tree_for_particles(n_particles);
        let pos = Vec2Length::meter(3.5, 3.5);
        let solver = Solver {
            opening_angle: Dimensionless::zero(),
            softening_length: Length::zero(),
        };
        let acc1 = solver.get_acceleration_on_particle(&tree, pos, Entity::from_raw(0));
        let acc2 = direct_sum(&solver, &pos, get_particles(n_particles).iter().collect());
        let relative_diff = (acc1 - acc2).length() / (acc1.length() + acc2.length());
        // Precision is pretty low with f32, so change this to f64 once variable precision is implemented
        assert!(relative_diff.value() < &1e-5);
    }

    fn direct_sum(
        solver: &Solver,
        pos1: &Vec2Length,
        other_positions: Vec<&LeafData>,
    ) -> Vec2Acceleration {
        let mut total = Vec2Acceleration::zero();
        for particle in other_positions.iter() {
            total += solver.get_gravity_acceleration(pos1, &particle.pos, particle.mass);
        }
        total
    }
}
