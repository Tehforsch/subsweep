use std::collections::HashMap;

use bevy::prelude::*;
use mpi::point_to_point::Status;
use mpi::traits::Communicator;
use mpi::traits::Equivalence;
use mpi::traits::Source;
use mpi::Rank;

use crate::mpi_world::MpiWorld;
use crate::position::Position;
use crate::units::f32::second;
use crate::units::vec2::Length;
use crate::velocity::Velocity;

#[derive(Equivalence)]
struct Timestep(crate::units::f32::Time);

#[derive(Clone)]
pub struct Domain {
    pub upper_left: Length,
    pub lower_right: Length,
}

impl Domain {
    fn contains(&self, pos: &Position) -> bool {
        let ul = self.upper_left.unwrap_value();
        let lr = self.lower_right.unwrap_value();
        let pos = pos.0.unwrap_value();
        ul.x <= pos.x && pos.x < lr.x && ul.y <= pos.y && pos.y < lr.y
    }
}

#[derive(Clone)]
pub struct DomainDistribution {
    pub domains: HashMap<Rank, Domain>,
}

pub struct PhysicsPlugin(pub DomainDistribution);

impl DomainDistribution {
    pub fn target_rank(&self, pos: &Position) -> Rank {
        *self
            .domains
            .iter()
            .find(|(_, domain)| domain.contains(pos))
            .map(|(rank, _)| rank)
            .unwrap_or(&0)
    }
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Timestep(second(0.001)))
            .insert_resource(self.0.clone())
            .add_system(exchange_particles_system)
            .add_system(integrate_motion_system);
    }
}

fn integrate_motion_system(mut query: Query<(&mut Position, &Velocity)>, timestep: Res<Timestep>) {
    for (mut pos, velocity) in query.iter_mut() {
        pos.0 += velocity.0 * timestep.0;
    }
}

#[derive(Equivalence)]
struct ParticleExchangeData {
    vel: Velocity,
    pos: Position,
}

fn exchange_particles_system(
    mut commands: Commands,
    particles: Query<(Entity, &Position, &Velocity)>,
    world: Res<MpiWorld>,
    domain: Res<DomainDistribution>,
) {
    let mut positions = vec![];
    let this_rank = world.rank();
    for (entity, pos, vel) in particles.iter() {
        let target_rank = domain.target_rank(pos);
        if target_rank != this_rank {
            commands.entity(entity).despawn();
            positions.push((
                target_rank,
                ParticleExchangeData {
                    pos: pos.clone(),
                    vel: vel.clone(),
                },
            ));
        }
    }
    for rank in world.other_ranks() {
        let num = positions
            .iter()
            .filter(|(new_rank, _)| *new_rank == rank)
            .count();
        world.send(rank, num);
        if num > 0 {
            println!(
                "{} coming from {} to {}",
                positions.len(),
                world.rank(),
                rank
            );
        }
    }
    for (rank, data) in positions.into_iter() {
        world.send(rank, data);
    }
    for rank in world.other_ranks() {
        let (num_incoming, status): (usize, Status) = world.world().process_at_rank(rank).receive();
        if num_incoming > 0 {
            println!(
                "{} incoming from {} to {}",
                num_incoming,
                rank,
                world.rank()
            );
            let (moved_to_own_domain, status): (Vec<ParticleExchangeData>, Status) =
                world.world().process_at_rank(rank).receive_vec();
            for data in moved_to_own_domain.into_iter() {
                commands.spawn().insert(data.pos).insert(data.vel);
            }
        }
    }
}
