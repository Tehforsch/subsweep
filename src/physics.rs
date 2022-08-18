use bevy::prelude::*;
use mpi::traits::Equivalence;

use crate::communication::ExchangeCommunicator;
use crate::communication::Rank;
use crate::domain::DomainDistribution;
use crate::get_domain_distribution;
use crate::mass::Mass;
use crate::particle::LocalParticleBundle;
use crate::position::Position;
use crate::units::f32::meter;
use crate::units::f32::newton;
use crate::units::f32::second;
use crate::units::vec2;
use crate::velocity::Velocity;

#[derive(Component)]
pub struct LocalParticle;

#[derive(Component)]
pub struct RemoteParticle(pub Rank);

#[derive(Equivalence)]
struct Timestep(crate::units::f32::Time);

pub struct Time(pub crate::units::f32::Time);

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        let rank = app.world.get_resource::<Rank>().unwrap();
        let domain_distribution = get_domain_distribution();
        let domain = domain_distribution.domains[&rank].clone();
        app.insert_resource(Timestep(second(0.01)))
            .insert_resource(Time(second(0.00)))
            .insert_resource(domain_distribution)
            .insert_resource(domain)
            .add_system(integrate_motion_system)
            .add_system(time_system)
            .add_system(spring_system)
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
    mass: Mass,
}

fn exchange_particles_system(
    mut commands: Commands,
    mut communicator: NonSendMut<ExchangeCommunicator<ParticleExchangeData>>,
    rank: Res<Rank>,
    domain: Res<DomainDistribution>,
    particles: Query<(Entity, &Position, &Velocity, &Mass), With<LocalParticle>>,
) {
    for (entity, pos, vel, mass) in particles.iter() {
        let target_rank = domain.target_rank(pos);
        if target_rank != *rank {
            commands.entity(entity).despawn();
            communicator.send(
                target_rank,
                ParticleExchangeData {
                    pos: pos.clone(),
                    vel: vel.clone(),
                    mass: mass.clone(),
                },
            );
        }
    }

    for (_, moved_to_own_domain) in communicator.receive_vec().into_iter() {
        for data in moved_to_own_domain.into_iter() {
            commands
                .spawn()
                .insert_bundle(LocalParticleBundle::new(data.pos, data.vel, data.mass));
        }
    }
}

fn time_system(mut time: ResMut<self::Time>, timestep: Res<Timestep>) {
    time.0 += timestep.0;
}

fn spring_system(
    timestep: Res<Timestep>,
    mut particles: Query<(&mut Velocity, &Position, &Mass), With<LocalParticle>>,
) {
    let spring_constant = newton(20.0) / meter(1.0);
    for (mut vel, pos, mass) in particles.iter_mut() {
        vel.0 -= (pos.0 - vec2::meter(Vec2::new(0.5, 0.5))) * timestep.0 * spring_constant / mass.0;
    }
}
