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
use crate::communication::SizedCommunicator;
use crate::physics::LocalParticle;
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

fn determine_global_extent_system(// mut commands: Commands,
    // particles: Query<&Position, With<LocalParticle>>,
) {
    debug!("TODO: Determine global extent");
}

fn domain_decomposition_system(
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
    let num_desired_particles_per_segment = particles.len() / NUM_DESIRED_SEGMENTS_PER_RANK;
    let segments = get_segments(&particles, num_desired_particles_per_segment);
    for rank in comm.other_ranks() {
        comm.send_vec(rank, segments.clone());
    }
    let all_segments = comm.receive_vec();
}
