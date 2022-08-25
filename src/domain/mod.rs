use bevy::prelude::*;
use mpi::traits::Equivalence;

pub mod extent;
mod peano_hilbert;
pub mod quadtree;
pub mod segment;

use self::extent::Extent;
use self::peano_hilbert::PeanoHilbertKey;
use self::segment::get_segments;
use self::segment::Segment;
use crate::communication::AllGatherCommunicator;
use crate::communication::AllReduceCommunicator;
use crate::communication::CollectiveCommunicator;
use crate::communication::ExchangeCommunicator;
use crate::communication::NumRanks;
use crate::communication::Rank;
use crate::communication::SizedCommunicator;
use crate::communication::SumCommunicator;
use crate::communication::WorldRank;
use crate::domain::segment::merge_and_split_segments;
use crate::mass::Mass;
use crate::particle::LocalParticleBundle;
use crate::physics::LocalParticle;
use crate::position::Position;
use crate::velocity::Velocity;

const NUM_DESIRED_SEGMENTS_PER_RANK: usize = 50;

#[derive(StageLabel)]
pub enum DomainDecompositionStages {
    Decomposition,
}

pub struct DomainDecompositionPlugin;

impl Plugin for DomainDecompositionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GlobalExtent(Extent::sentinel()));
        app.add_stage_after(
            CoreStage::Update,
            DomainDecompositionStages::Decomposition,
            SystemStage::parallel(),
        );
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

#[derive(Debug)]
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
    mut extent_communicator: NonSendMut<AllGatherCommunicator<Extent>>,
    mut global_extent: ResMut<GlobalExtent>,
) {
    let extent =
        Extent::from_positions(particles.iter().map(|x| &x.0)).unwrap_or(Extent::sentinel());
    let all_extents = (*extent_communicator).all_gather(&extent);
    *global_extent = GlobalExtent(
        Extent::get_all_encompassing(all_extents.iter())
            .expect("Failed to find simulation extents - are there no particles?")
            .pad(),
    );
}

fn get_sorted_peano_hilbert_keys(
    extent: &Extent,
    particles: &Query<(Entity, &Position), With<LocalParticle>>,
) -> Vec<ParticleData> {
    let mut particles: Vec<_> = particles
        .iter()
        .map(|(entity, pos)| ParticleData {
            entity,
            key: PeanoHilbertKey::new(extent, &pos.0),
        })
        .collect();
    particles.sort();
    particles
}

fn get_total_number_particles(
    num: usize,
    communicator: &mut AllReduceCommunicator<usize>,
) -> usize {
    communicator.collective_sum(&num)
}

fn get_global_segments_from_peano_hilbert_keys(
    segment_communicator: &mut ExchangeCommunicator<Segment>,
    particles: &Vec<ParticleData>,
    num_particles_total: usize,
) -> Vec<Segment> {
    let num_desired_particles_per_segment =
        num_particles_total / (NUM_DESIRED_SEGMENTS_PER_RANK * segment_communicator.size());
    let segments = get_segments(&particles, num_desired_particles_per_segment);
    for rank in segment_communicator.other_ranks() {
        segment_communicator.send_vec(rank, segments.clone());
    }
    // replace this with an appropriate allgather call at some point
    let mut all_segments = segment_communicator.receive_vec();
    all_segments.insert(segment_communicator.rank(), segments);
    merge_and_split_segments(all_segments, num_desired_particles_per_segment)
}

fn find_key_cutoffs(
    num_ranks: usize,
    num_particles_total: usize,
    global_segment_list: Vec<Segment>,
) -> Vec<PeanoHilbertKey> {
    let load_per_rank = num_particles_total / num_ranks;
    let mut load = 0;
    let mut key_cutoffs_by_rank = vec![];
    for segment in global_segment_list.into_iter() {
        load += segment.num_particles;
        if load >= load_per_rank {
            key_cutoffs_by_rank.push(segment.end());
            if key_cutoffs_by_rank.len() == num_ranks - 1 {
                break;
            }
            load = 0;
        }
    }
    key_cutoffs_by_rank
}

fn domain_decomposition_system(
    mut commands: Commands,
    mut segment_communicator: NonSendMut<ExchangeCommunicator<Segment>>,
    mut exchange_communicator: NonSendMut<ExchangeCommunicator<ParticleExchangeData>>,
    mut num_particle_communicator: NonSendMut<AllReduceCommunicator<usize>>,
    rank: Res<WorldRank>,
    num_ranks: Res<NumRanks>,
    extent: Res<GlobalExtent>,
    particles: Query<(Entity, &Position), With<LocalParticle>>,
    full_particle_data: Query<(&Position, &Velocity, &Mass), With<LocalParticle>>,
) {
    let particles = get_sorted_peano_hilbert_keys(&extent.0, &particles);
    let num_particles_total =
        get_total_number_particles(particles.len(), &mut num_particle_communicator);
    let global_segment_list = get_global_segments_from_peano_hilbert_keys(
        &mut segment_communicator,
        &particles,
        num_particles_total,
    );
    let key_cutoffs_by_rank =
        find_key_cutoffs(num_ranks.0, num_particles_total, global_segment_list);
    let target_rank = |key: &PeanoHilbertKey| {
        key_cutoffs_by_rank
            .binary_search(&key)
            .unwrap_or_else(|e| e) as Rank
    };
    for ParticleData { key, entity } in particles.iter() {
        let target_rank = target_rank(key);
        if target_rank != rank.0 {
            commands.entity(*entity).despawn();
            let (pos, vel, mass) = full_particle_data.get(*entity).unwrap();
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
