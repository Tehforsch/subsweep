mod gravity;
mod parameters;

use bevy::prelude::*;
pub use gravity::QuadTree;
use mpi::traits::Equivalence;

use self::gravity::construct_quad_tree_system;
use self::gravity::gravity_system;
use self::parameters::Parameters;
use crate::communication::ExchangeCommunicator;
use crate::communication::Rank;
use crate::domain::quadtree::QuadTreeConfig;
use crate::domain::DomainDistribution;
use crate::mass::Mass;
use crate::parameters::ParameterPlugin;
use crate::particle::LocalParticleBundle;
use crate::position::Position;
use crate::units;
use crate::velocity::Velocity;

#[derive(Component)]
pub struct LocalParticle;

#[derive(Component)]
pub struct RemoteParticle(pub Rank);

#[derive(Equivalence)]
struct Timestep(crate::units::Time);

pub struct Time(pub crate::units::Time);

pub struct PhysicsPlugin;

// Cannot wait for stageless
#[derive(StageLabel)]
pub enum PhysicsStages {
    QuadTreeConstruction,
    Gravity,
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_after(
            CoreStage::Update,
            PhysicsStages::QuadTreeConstruction,
            SystemStage::parallel(),
        );
        app.add_stage_after(
            PhysicsStages::QuadTreeConstruction,
            PhysicsStages::Gravity,
            SystemStage::parallel(),
        );
        app.add_plugin(ParameterPlugin::<Parameters>::new("physics"))
            .add_plugin(ParameterPlugin::<QuadTreeConfig>::new("tree"))
            .insert_resource(Timestep(units::Time::second(0.01)))
            .insert_resource(Time(units::Time::second(0.00)))
            .add_system_to_stage(
                PhysicsStages::QuadTreeConstruction,
                construct_quad_tree_system,
            )
            .add_system_to_stage(PhysicsStages::Gravity, gravity_system)
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
        let target_rank = domain.target_rank(&pos.0);
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
