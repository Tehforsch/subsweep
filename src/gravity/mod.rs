use bevy::prelude::*;
use mpi::traits::Equivalence;

use crate::communication::DataByRank;
use crate::communication::ExchangeCommunicator;
use crate::communication::Identified;
use crate::communication::WorldRank;
use crate::components::Position;
use crate::components::Velocity;
use crate::domain::TopLevelIndices;
use crate::parameters::SimulationBox;
use crate::parameters::TimestepParameters;
use crate::prelude::Particles;
use crate::quadtree::Node;
use crate::quadtree::*;
use crate::units;
use crate::units::Dimensionless;
use crate::units::Length;
use crate::units::VecAcceleration;
use crate::units::VecLength;
use crate::units::GRAVITY_CONSTANT;

pub(super) mod mass_moments;
mod parameters;
pub(super) mod plugin;
mod quadtree;
#[cfg(test)]
pub(crate) mod tests;

pub use mass_moments::MassMoments;
pub use parameters::GravityParameters;
pub use plugin::GravityPlugin;
pub use quadtree::LeafData;
pub use quadtree::NodeData;
pub use quadtree::QuadTree;

struct Solver {
    softening_length: Length,
    opening_angle: Dimensionless,
    box_: SimulationBox,
}

impl Solver {
    pub fn new(parameters: &GravityParameters, box_: &SimulationBox) -> Self {
        Self {
            softening_length: parameters.softening_length,
            opening_angle: parameters.opening_angle,
            box_: box_.clone(),
        }
    }
}

impl Solver {
    fn calc_gravity_acceleration(
        &self,
        pos1: &VecLength,
        pos2: &VecLength,
        mass2: units::Mass,
    ) -> VecAcceleration {
        let distance_vector = self.box_.periodic_distance_vec(pos1, pos2);
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
                .map(|particle| self.calc_gravity_acceleration(pos, &particle.pos, particle.mass))
                .sum(),
        }
    }

    fn should_be_opened(&self, child: &QuadTree, pos: &VecLength) -> bool {
        let distance = self.box_.periodic_distance(pos, &child.extent.center());
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
    tree: Res<QuadTree>,
    world_rank: Res<WorldRank>,
    indices: Res<TopLevelIndices>,
    mut particles: Particles<(Entity, &Position, &mut Velocity)>,
    parameters: Res<GravityParameters>,
    mut request_comm: ExchangeCommunicator<Identified<GravityCalculationRequest>>,
    mut reply_comm: ExchangeCommunicator<Identified<GravityCalculationReply>>,
    box_: Res<SimulationBox>,
    timestep: Res<TimestepParameters>,
) {
    let gravity = Solver::new(&parameters, &box_);
    let mut outgoing_requests = DataByRank::from_communicator(&*request_comm);
    let add_acceleration = |vel: &mut Velocity, acceleration| {
        **vel += acceleration * timestep.max_timestep;
    };
    for (entity, pos, mut vel) in particles.iter_mut() {
        for (rank, index) in indices.flat_iter() {
            let sub_tree = &tree[index];
            if rank == **world_rank {
                add_acceleration(&mut vel, gravity.traverse_tree(sub_tree, pos));
            } else if gravity.should_be_opened(sub_tree, pos) {
                outgoing_requests.push(
                    rank,
                    Identified::new(
                        entity,
                        GravityCalculationRequest {
                            index: *index,
                            pos: *pos.clone(),
                        },
                    ),
                );
            } else {
                add_acceleration(
                    &mut vel,
                    gravity.calc_gravity_acceleration_for_moments(pos, &sub_tree.data.moments),
                );
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
