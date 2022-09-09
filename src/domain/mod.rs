use bevy::prelude::*;
use mpi::traits::Equivalence;

mod exchange_data_plugin;
mod extent;
mod peano_hilbert;
pub mod quadtree;
mod segment;

pub use self::exchange_data_plugin::ExchangeDataPlugin;
use self::exchange_data_plugin::OutgoingEntities;
pub use self::extent::Extent;
use self::peano_hilbert::PeanoHilbertKey;
use self::segment::get_position;
use self::segment::get_segments;
pub use self::segment::Segment;
use crate::communication::AllGatherCommunicator;
use crate::communication::AllReduceCommunicator;
use crate::communication::CollectiveCommunicator;
use crate::communication::CommunicationPlugin;
use crate::communication::CommunicationType;
use crate::communication::ExchangeCommunicator;
use crate::communication::Rank;
use crate::communication::SizedCommunicator;
use crate::communication::SumCommunicator;
use crate::communication::WorldRank;
use crate::communication::WorldSize;
use crate::domain::segment::merge_and_split_segments;
use crate::mass;
use crate::physics::LocalParticle;
use crate::physics::MassMoments;
use crate::position::Position;
use crate::units::Mass;
use crate::velocity::Velocity;
use crate::visualization::get_color;
use crate::visualization::DrawRect;

const NUM_DESIRED_SEGMENTS_PER_RANK: usize = 50;

#[derive(Debug, Clone)]
pub struct AssignedSegment {
    pub segment: Segment,
    pub rank: Rank,
    pub extent: Option<Extent>,
    pub mass: MassMoments,
}

#[derive(Clone, Debug, Equivalence)]
struct SegmentCommunicationData {
    index: usize,
    extent: Extent,
    valid_extent: bool,
    mass: MassMoments,
}

#[derive(Default)]
pub struct Segments(pub Vec<AssignedSegment>);

#[derive(StageLabel)]
pub enum DomainDecompositionStages {
    Decomposition,
    Exchange,
    AfterExchange,
}

pub struct DomainDecompositionPlugin;

impl Plugin for DomainDecompositionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GlobalExtent(Extent::sentinel()))
            .insert_resource(Segments::default());
        app.add_stage_after(
            CoreStage::Update,
            DomainDecompositionStages::Decomposition,
            SystemStage::parallel(),
        );
        app.add_stage_after(
            DomainDecompositionStages::Decomposition,
            DomainDecompositionStages::Exchange,
            SystemStage::parallel(),
        );
        app.add_stage_after(
            DomainDecompositionStages::Exchange,
            DomainDecompositionStages::AfterExchange,
            SystemStage::parallel(),
        );
        app.add_system_to_stage(
            DomainDecompositionStages::Decomposition,
            determine_global_extent_system,
        );
        app.add_system_to_stage(
            DomainDecompositionStages::Decomposition,
            domain_decomposition_system.after(determine_global_extent_system),
        )
        .add_system_to_stage(
            DomainDecompositionStages::AfterExchange,
            communicate_segment_extent_system,
        )
        .add_plugin(CommunicationPlugin::<Extent>::new(
            CommunicationType::AllGather,
        ))
        .add_plugin(CommunicationPlugin::<usize>::new(CommunicationType::Sum))
        .add_plugin(CommunicationPlugin::<SegmentCommunicationData>::new(
            CommunicationType::AllGather,
        ))
        .add_plugin(CommunicationPlugin::<Segment>::new(
            CommunicationType::Exchange,
        ));
    }
}

#[derive(Debug)]
pub struct GlobalExtent(pub Extent);

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
    particles: Query<&Position>,
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
    let num_desired_particles_per_segment = (num_particles_total
        / (NUM_DESIRED_SEGMENTS_PER_RANK * segment_communicator.size()))
    .max(1);
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
    global_segment_list: &[Segment],
) -> Vec<PeanoHilbertKey> {
    let load_per_rank = num_particles_total / num_ranks;
    let mut load = 0;
    let mut key_cutoffs_by_rank = vec![];
    for segment in global_segment_list.iter() {
        load += segment.num_particles;
        if load >= load_per_rank {
            // We want the keys to be inclusive, so that
            // the range of keys for a given rank is given by the half open interval
            // [key_cutoff[i], key_cutoff[i+1])
            // so we need to subtract one here.
            key_cutoffs_by_rank.push(segment.end().prev());
            if key_cutoffs_by_rank.len() == num_ranks - 1 {
                break;
            }
            load = 0;
        }
    }
    key_cutoffs_by_rank
}

fn domain_decomposition_system(
    mut segment_communicator: NonSendMut<ExchangeCommunicator<Segment>>,
    mut num_particle_communicator: NonSendMut<AllReduceCommunicator<usize>>,
    mut outgoing_entities: ResMut<OutgoingEntities>,
    mut segments: ResMut<Segments>,
    rank: Res<WorldRank>,
    num_ranks: Res<WorldSize>,
    extent: Res<GlobalExtent>,
    particles: Query<(Entity, &Position), With<LocalParticle>>,
) {
    let particles = get_sorted_peano_hilbert_keys(&extent.0, &particles);
    let num_particles_total =
        get_total_number_particles(particles.len(), &mut num_particle_communicator);
    let segment_list = get_global_segments_from_peano_hilbert_keys(
        &mut segment_communicator,
        &particles,
        num_particles_total,
    );
    let key_cutoffs_by_rank = find_key_cutoffs(num_ranks.0, num_particles_total, &segment_list);
    let target_rank = |key: &PeanoHilbertKey| {
        key_cutoffs_by_rank
            .binary_search(&key)
            .map(|found| found)
            .unwrap_or_else(|e| e) as Rank
    };
    for ParticleData { key, entity } in particles.iter() {
        let target_rank = target_rank(key);
        if target_rank != rank.0 {
            outgoing_entities.add(target_rank, *entity);
        }
    }
    segments.0 = segment_list
        .iter()
        .map(|segment| {
            let rank = target_rank(&segment.start());
            debug_assert_eq!(
                target_rank(&segment.start()),
                target_rank(&PeanoHilbertKey(segment.end().0 - 1))
            );
            AssignedSegment {
                segment: segment.clone(),
                rank: rank,
                extent: None,
                mass: MassMoments::default(),
            }
        })
        .collect()
}

fn get_extent_and_mass_of_segment(
    particles: &Query<(Entity, &mass::Mass, &Position), With<LocalParticle>>,
    keys: &[ParticleData],
    segment: &AssignedSegment,
) -> (Option<Extent>, MassMoments) {
    let start = get_position(&keys, |p: &ParticleData| p.key, &segment.segment.start());
    let end = get_position(&keys, |p: &ParticleData| p.key, &segment.segment.end());
    let extent = Extent::from_positions_allow_empty(
        keys[start..end]
            .iter()
            .map(|p| &particles.get(p.entity).unwrap().2 .0),
    );
    let mass = keys[start..end]
        .iter()
        .map(|p| {
            let (_, mass, pos) = particles.get(p.entity).unwrap();
            (mass.0, pos.0)
        })
        .sum();
    (extent, mass)
}

fn communicate_segment_extent_system(
    mut segment_extent_communicator: NonSendMut<AllGatherCommunicator<SegmentCommunicationData>>,
    mut segments: ResMut<Segments>,
    particles: Query<(Entity, &Position), With<LocalParticle>>,
    particles_with_mass: Query<(Entity, &mass::Mass, &Position), With<LocalParticle>>,
    rank: Res<WorldRank>,
    world_size: Res<WorldSize>,
    extent: Res<GlobalExtent>,
) {
    let keys = get_sorted_peano_hilbert_keys(&extent.0, &particles);
    let extents: Vec<_> = segments
        .0
        .iter()
        .enumerate()
        .filter(|(_, segment)| segment.rank == rank.0)
        .map(|(index, segment)| {
            let (extent, mass) =
                get_extent_and_mass_of_segment(&particles_with_mass, &keys, segment);
            let (extent, valid_extent) = match extent {
                Some(extent) => (extent, true),
                None => (Extent::sentinel(), false),
            };
            SegmentCommunicationData {
                index,
                extent,
                valid_extent,
                mass,
            }
        })
        .collect();
    let mut num_segments_by_rank = vec![0i32; world_size.0];
    // This is slow and unnecessarily O(n^2) but it probably doesn't
    // matter performance wise - replace if ever necessary
    for rank in 0..world_size.0 {
        num_segments_by_rank[rank] = segments
            .0
            .iter()
            .filter(|segment| segment.rank == rank as i32)
            .count() as i32;
    }
    let all = (*segment_extent_communicator).all_gather_varcount(&extents, &num_segments_by_rank);
    for seg_data in all {
        if seg_data.valid_extent {
            segments.0[seg_data.index].extent = Some(seg_data.extent);
        }
        segments.0[seg_data.index].mass = seg_data.mass;
    }
}

#[derive(Component)]
pub struct SegmentOutline;

pub fn show_segment_extent_system(
    mut commands: Commands,
    segments: Res<Segments>,
    outlines: Query<Entity, With<SegmentOutline>>,
) {
    for entity in outlines.iter() {
        commands.entity(entity).despawn();
    }
    for seg in segments.0.iter() {
        match seg.extent.as_ref() {
            Some(extent) => {
                commands.spawn().insert(SegmentOutline).insert(DrawRect {
                    lower_left: extent.min,
                    upper_right: extent.max,
                    color: get_color(seg.rank),
                });
            }
            _ => {}
        }
    }
}

#[derive(Equivalence, Clone)]
pub(super) struct ParticleExchangeData {
    vel: Velocity,
    pos: Position,
    mass: Mass,
}
