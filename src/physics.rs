use std::collections::HashMap;

use bevy::prelude::*;
use mpi::traits::Equivalence;
use mpi::Rank;

use crate::communication::BufferedCommunicator;
use crate::mpi_world::MpiCommunicator;
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
        app.insert_resource(Timestep(second(0.01)))
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
    let mut communicator = MpiCommunicator::new(&world);
    let this_rank = world.rank();
    for (entity, pos, vel) in particles.iter() {
        let target_rank = domain.target_rank(pos);
        if target_rank != this_rank {
            commands.entity(entity).despawn();
            communicator.send(
                target_rank,
                ParticleExchangeData {
                    pos: pos.clone(),
                    vel: vel.clone(),
                },
            );
        }
    }

    for (_, moved_to_own_domain) in communicator.receive_vec().into_iter() {
        for data in moved_to_own_domain.into_iter() {
            commands.spawn().insert(data.pos).insert(data.vel);
        }
    }
}
