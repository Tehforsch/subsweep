use bevy::prelude::*;
use mpi::traits::Equivalence;

use super::parameters::Parameters;
use super::LocalParticle;
use super::MassMoments;
use super::Timestep;
use crate::communication::DataByRank;
use crate::communication::ExchangeCommunicator;
use crate::communication::Identified;
use crate::communication::WorldRank;
use crate::domain::quadtree::Node;
use crate::domain::quadtree::QuadTree;
use crate::domain::quadtree::QuadTreeIndex;
use crate::domain::TopLevelIndices;
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
    fn calc_gravity_acceleration(
        &self,
        pos1: &VecLength,
        pos2: &VecLength,
        mass2: units::Mass,
    ) -> VecAcceleration {
        let distance_vector = *pos1 - *pos2;
        let distance = distance_vector.length() + self.softening_length;
        -distance_vector * GRAVITY_CONSTANT * mass2 / distance.cubed()
    }

    pub fn calc_gravity_acceleration_for_moments(
        &self,
        pos: &VecLength,
        moments: &MassMoments,
    ) -> VecAcceleration {
        self.calc_gravity_acceleration(pos, &moments.center_of_mass(), moments.total())
    }

    pub fn traverse_tree(&self, tree: &QuadTree, pos: &VecLength) -> VecAcceleration {
        match tree.node {
            Node::Tree(ref children) => children
                .iter()
                .map(|child| {
                    if self.should_be_opened(child, pos) {
                        self.traverse_tree(child, pos)
                    } else {
                        self.calc_gravity_acceleration_for_moments(pos, &child.data.moments)
                    }
                })
                .sum(),
            Node::Leaf(ref leaf) => leaf
                .iter()
                .map(|particle| self.calc_gravity_acceleration(&pos, &particle.pos, particle.mass))
                .sum(),
        }
    }

    fn should_be_opened(&self, child: &QuadTree, pos: &VecLength) -> bool {
        let distance = pos.distance(&child.extent.center());
        let length = child.extent.max_side_length();
        length / distance > self.opening_angle
    }
}

#[derive(Equivalence, Debug)]
pub(super) struct GravityCalculationRequest {
    pos: VecLength,
    index: QuadTreeIndex,
}

#[derive(Equivalence, Debug)]
pub(super) struct GravityCalculationReply {
    acc: VecAcceleration,
}

pub(super) fn gravity_system(
    timestep: Res<Timestep>,
    tree: Res<QuadTree>,
    world_rank: Res<WorldRank>,
    indices: Res<TopLevelIndices>,
    mut particles: Query<(Entity, &Position, &mut Velocity), With<LocalParticle>>,
    parameters: Res<Parameters>,
    mut request_comm: NonSendMut<ExchangeCommunicator<Identified<GravityCalculationRequest>>>,
    mut reply_comm: NonSendMut<ExchangeCommunicator<Identified<GravityCalculationReply>>>,
) {
    let gravity = Solver {
        softening_length: parameters.softening_length,
        opening_angle: parameters.opening_angle,
    };
    let mut outgoing_requests = DataByRank::from_communicator(&*request_comm);
    let add_acceleration = |vel: &mut Velocity, acceleration| {
        **vel += acceleration * **timestep;
    };
    for (entity, pos, mut vel) in particles.iter_mut() {
        for (rank, index) in indices.flat_iter() {
            let sub_tree = &tree[index];
            if rank == **world_rank {
                add_acceleration(&mut vel, gravity.traverse_tree(sub_tree, &**pos));
            } else {
                if gravity.should_be_opened(sub_tree, pos) {
                    outgoing_requests.push(
                        rank,
                        Identified::new(
                            entity,
                            GravityCalculationRequest {
                                index: index.clone(),
                                pos: *pos.clone(),
                            },
                        ),
                    );
                } else {
                    add_acceleration(
                        &mut vel,
                        gravity
                            .calc_gravity_acceleration_for_moments(&**pos, &sub_tree.data.moments),
                    );
                }
            }
        }
    }
    let num_outgoing_requests = outgoing_requests.size();
    let incoming_requests = request_comm.exchange_all(outgoing_requests);
    let mut result = DataByRank::from_communicator(&*reply_comm);
    for (rank, requests) in incoming_requests {
        for request in requests {
            let tree = &tree[&request.data.index];
            let acc = gravity.traverse_tree(tree, &request.data.pos);
            result.push(
                rank,
                Identified {
                    key: request.key,
                    data: GravityCalculationReply { acc },
                },
            );
        }
    }
    let accelerations = reply_comm.exchange_all(result);
    assert_eq!(accelerations.size(), num_outgoing_requests);
    for (_, accelerations) in accelerations.iter() {
        for acc in accelerations {
            let entity = acc.entity();
            let (_, _, mut vel) = particles.get_mut(entity).unwrap();
            add_acceleration(&mut vel, acc.data.acc);
        }
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
    use crate::units::DVec2Acceleration;
    use crate::units::DVec2Length;
    use crate::units::Dimensionless;
    use crate::units::Length;
    use crate::units::Mass;

    fn get_particles(n: i32) -> Vec<LeafData> {
        (1..n)
            .flat_map(move |x| {
                (1..n).map(move |y| LeafData {
                    entity: Entity::from_raw((x * n + y) as u32),
                    pos: DVec2Length::meter(x as f64, y as f64),
                    mass: Mass::kilogram(x as f64 * y as f64),
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
        let pos = DVec2Length::meter(3.5, 3.5);
        let solver = Solver {
            opening_angle: Dimensionless::zero(),
            softening_length: Length::zero(),
        };
        let acc1 = solver.traverse_tree(&tree, &pos);
        let acc2 = direct_sum(&solver, &pos, get_particles(n_particles).iter().collect());
        let relative_diff = (acc1 - acc2).length() / (acc1.length() + acc2.length());
        // Precision is pretty low with f64, so change this to f64 once variable precision is implemented
        assert!(relative_diff.value() < &1e-5);
    }

    fn direct_sum(
        solver: &Solver,
        pos1: &DVec2Length,
        other_positions: Vec<&LeafData>,
    ) -> DVec2Acceleration {
        let mut total = DVec2Acceleration::zero();
        for particle in other_positions.iter() {
            total += solver.calc_gravity_acceleration(pos1, &particle.pos, particle.mass);
        }
        total
    }
}
