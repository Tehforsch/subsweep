use bevy::prelude::*;
use mpi::traits::Equivalence;

mod extent;
mod peano_hilbert;
pub mod quadtree;
pub mod segment;

use self::extent::Extent;
use self::peano_hilbert::PeanoHilbertKey;
use self::segment::get_segments;
use self::segment::Segment;
use crate::communication::ExchangeCommunicator;
use crate::communication::Rank;
use crate::communication::SizedCommunicator;
use crate::domain::segment::merge_and_split_segments;
use crate::mass::Mass;
use crate::particle::LocalParticleBundle;
use crate::physics::LocalParticle;
use crate::position::Position;
use crate::units::Length;
use crate::units::VecLength;
use crate::velocity::Velocity;

#[derive(StageLabel)]
pub enum DomainDecompositionStages {
    Decomposition,
}

pub struct DomainDecompositionPlugin;

impl Plugin for DomainDecompositionPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_after(
            CoreStage::Update,
            DomainDecompositionStages::Decomposition,
            SystemStage::parallel(),
        );
        let extent = Extent::new(
            Length::meter(-100.0),
            Length::meter(100.0),
            Length::meter(-100.0),
            Length::meter(100.0),
        );
        app.insert_resource(GlobalExtent(extent));
        app.add_system_to_stage(
            DomainDecompositionStages::Decomposition,
            determine_global_extent_system,
        );
        app.add_system_to_stage(
            DomainDecompositionStages::Decomposition,
            domain_decomposition_system.after(determine_global_extent_system),
        );
    }
}

struct GlobalExtent(Extent);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ParticleData {
    key: PeanoHilbertKey,
    entity: Entity,
}

impl ParticleData {
    fn key(&self) -> PeanoHilbertKey {
        self.key
    }
}

fn determine_global_extent_system(
    particles: Query<&Position, With<LocalParticle>>,
    mut extent_communicator: NonSendMut<ExchangeCommunicator<Extent>>,
) {
    let extents = Extent::from_positions(particles.iter().map(|x| &x.0));
    debug!("TODO: Determine global extent");
}

fn domain_decomposition_system(
    mut commands: Commands,
    mut segment_communicator: NonSendMut<ExchangeCommunicator<Segment>>,
    mut exchange_communicator: NonSendMut<ExchangeCommunicator<ParticleExchangeData>>,
    rank: Res<Rank>,
    extent: Res<GlobalExtent>,
    particles: Query<(Entity, &Position), With<LocalParticle>>,
    full_particles: Query<(Entity, &Position, &Velocity, &Mass), With<LocalParticle>>,
) {
    let mut particles: Vec<_> = particles
        .iter()
        .map(|(entity, pos)| ParticleData {
            entity,
            key: PeanoHilbertKey::new(&extent.0, &pos.0),
        })
        .collect();
    particles.sort();
    const NUM_DESIRED_SEGMENTS_PER_RANK: usize = 50;
    let num_desired_particles_per_segment = particles.len() / NUM_DESIRED_SEGMENTS_PER_RANK;
    let segments = get_segments(&particles, num_desired_particles_per_segment);
    for rank in segment_communicator.other_ranks() {
        segment_communicator.send_vec(rank, segments.clone());
    }
    let mut all_segments = segment_communicator.receive_vec();
    all_segments.insert(*rank, segments);
    let total_load: usize = all_segments
        .iter()
        .map(|(_, segments)| segments.iter().map(|s| s.num_particles).sum::<usize>())
        .sum();
    let num_desired_particles_per_segment =
        total_load / (NUM_DESIRED_SEGMENTS_PER_RANK * segment_communicator.size());
    let all_segments = merge_and_split_segments(all_segments, num_desired_particles_per_segment);
    let load_per_rank = total_load / segment_communicator.size();
    let mut load = 0;
    let mut key_cutoffs_by_rank = vec![];
    for segment in all_segments.into_iter() {
        load += segment.num_particles;
        if load >= load_per_rank {
            key_cutoffs_by_rank.push(segment.end());
            if key_cutoffs_by_rank.len() == segment_communicator.size() - 1 {
                break;
            }
            load = 0;
        }
    }

    let target_rank = |pos: &VecLength| {
        let key = PeanoHilbertKey::new(&extent.0, &pos);
        key_cutoffs_by_rank
            .binary_search(&key)
            .unwrap_or_else(|e| e) as Rank
    };
    for (entity, pos, vel, mass) in full_particles.iter() {
        let target_rank = target_rank(&pos.0);
        if target_rank != *rank {
            commands.entity(entity).despawn();
            exchange_communicator.send(
                target_rank,
                ParticleExchangeData {
                    pos: pos.clone(),
                    vel: vel.clone(),
                    mass: mass.clone(),
                },
            );
        }
    }

    for (_, moved_to_own_domain) in exchange_communicator.receive_vec().into_iter() {
        for data in moved_to_own_domain.into_iter() {
            commands
                .spawn()
                .insert_bundle(LocalParticleBundle::new(data.pos, data.vel, data.mass));
        }
    }
}

#[derive(Equivalence, Clone)]
pub struct ParticleExchangeData {
    vel: Velocity,
    pos: Position,
    mass: Mass,
}
