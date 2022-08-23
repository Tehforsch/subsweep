use bevy::prelude::*;

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
use crate::domain::segment::sort_and_merge_segments;
use crate::physics::LocalParticle;
use crate::physics::RemoteParticle;
use crate::position::Position;
use crate::units::Length;

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

fn determine_global_extent_system(// mut commands: Commands,
    // particles: Query<&Position, With<LocalParticle>>,
) {
    debug!("TODO: Determine global extent");
}

fn domain_decomposition_system(
    mut commands: Commands,
    rank: Res<Rank>,
    extent: Res<GlobalExtent>,
    particles: Query<(Entity, &Position), With<LocalParticle>>,
    mut comm: NonSendMut<ExchangeCommunicator<Segment>>,
) {
    let mut particles: Vec<_> = particles
        .iter()
        .map(|(entity, pos)| ParticleData {
            entity,
            key: PeanoHilbertKey::new(&extent.0, &pos.0),
        })
        .collect();
    particles.sort();
    const NUM_DESIRED_SEGMENTS_PER_RANK: usize = 10;
    let num_desired_particles_per_segment =
        (particles.len() / NUM_DESIRED_SEGMENTS_PER_RANK).max(1);
    let segments = get_segments(&particles, num_desired_particles_per_segment);
    for rank in comm.other_ranks() {
        comm.send_vec(rank, segments.clone());
    }
    let mut all_segments = comm.receive_vec();
    all_segments.insert(*rank, segments);
    let all_segments = sort_and_merge_segments(all_segments);
    let total_load: usize = all_segments.iter().map(|s| s.num_particles).sum();
    let load_per_rank = total_load / comm.size();
    let mut load = 0;
    let mut current_rank = 0;
    for segment in all_segments.into_iter() {
        for part in segment.iter_contained_particles(&particles) {
            if current_rank != *rank {
                commands
                    .entity(part.entity)
                    .insert(RemoteParticle(current_rank));
            }
        }
        load += segment.num_particles;
        if load > load_per_rank {
            load = 0;
            current_rank += 1;
        }
    }
}
