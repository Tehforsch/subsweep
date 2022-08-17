use bevy::prelude::*;
use mpi::traits::Equivalence;

use crate::communication::ExchangeCommunicator;
use crate::communication::Rank;
use crate::communication::SizedCommunicator;
use crate::domain::DomainDistribution;
use crate::particle::LocalParticleBundle;
use crate::position::Position;
use crate::units::f32::second;
use crate::velocity::Velocity;

#[derive(Component)]
pub struct LocalParticle;

#[derive(Component)]
pub struct RemoteParticle(pub Rank);

#[derive(Equivalence)]
struct Timestep(crate::units::f32::Time);

pub struct Time(pub crate::units::f32::Time);

pub struct PhysicsPlugin(pub DomainDistribution);

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Timestep(second(0.01)))
            .insert_resource(Time(second(0.00)))
            .insert_resource(self.0.clone())
            .add_system(integrate_motion_system)
            .add_system(time_system)
            .add_system(exchange_particles_system.after(integrate_motion_system));
    }
}

fn integrate_motion_system(mut query: Query<(&mut Position, &Velocity)>, timestep: Res<Timestep>) {
    for (mut pos, velocity) in query.iter_mut() {
        pos.0 += velocity.0 * timestep.0;
    }
}

#[derive(Equivalence, Clone)]
pub struct ParticleExchangeData {
    vel: Velocity,
    pos: Position,
}

fn exchange_particles_system(
    mut commands: Commands,
    particles: Query<(Entity, &Position, &Velocity), With<LocalParticle>>,
    mut communicator: NonSendMut<ExchangeCommunicator<ParticleExchangeData>>,
    domain: Res<DomainDistribution>,
) {
    for (entity, pos, vel) in particles.iter() {
        let target_rank = domain.target_rank(pos);
        if target_rank != communicator.rank() {
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
            commands
                .spawn()
                .insert_bundle(LocalParticleBundle::new(data.pos, data.vel));
        }
    }
}

fn time_system(mut time: ResMut<self::Time>, timestep: Res<Timestep>) {
    time.0 += timestep.0;
}
