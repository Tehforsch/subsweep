use bevy::prelude::*;
use mpi::traits::Equivalence;

mod exchange_data_plugin;
mod extent;
// mod peano_hilbert;
pub mod quadtree;

pub use self::exchange_data_plugin::ExchangeDataPlugin;
use self::extent::Extent;
use self::quadtree::QuadTree;
use self::quadtree::QuadTreeConfig;
use crate::communication::AllGatherCommunicator;
use crate::communication::CollectiveCommunicator;
use crate::communication::CommunicationPlugin;
use crate::communication::CommunicationType;
use crate::mass::Mass;
use crate::position::Position;
use crate::velocity::Velocity;

#[derive(StageLabel)]
pub enum DomainDecompositionStages {
    TopLevelTreeConstruction,
    Decomposition,
    Exchange,
}

pub struct DomainDecompositionPlugin;

impl Plugin for DomainDecompositionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GlobalExtent(Extent::sentinel()));
        app.add_stage_after(
            CoreStage::Update,
            DomainDecompositionStages::TopLevelTreeConstruction,
            SystemStage::parallel(),
        );
        app.add_stage_after(
            DomainDecompositionStages::TopLevelTreeConstruction,
            DomainDecompositionStages::Decomposition,
            SystemStage::parallel(),
        );
        app.add_stage_after(
            DomainDecompositionStages::Decomposition,
            DomainDecompositionStages::Exchange,
            SystemStage::parallel(),
        );
        app.add_system_to_stage(
            DomainDecompositionStages::TopLevelTreeConstruction,
            determine_global_extent_system,
        )
        .add_system_to_stage(
            DomainDecompositionStages::TopLevelTreeConstruction,
            construct_quad_tree_system.after(determine_global_extent_system),
        )
        .add_system_to_stage(
            DomainDecompositionStages::Decomposition,
            domain_decomposition_system,
        )
        .add_plugin(CommunicationPlugin::<Extent>::new(
            CommunicationType::AllGather,
        ))
        .add_plugin(CommunicationPlugin::<usize>::new(CommunicationType::Sum));
    }
}

#[derive(Debug, Deref, DerefMut)]
struct GlobalExtent(Extent);

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
            .expect("Failed to find simulation extent - are there no particles?")
            .pad(),
    );
}

#[derive(Equivalence, Clone)]
pub(super) struct ParticleExchangeData {
    vel: Velocity,
    pos: Position,
    mass: Mass,
}

fn construct_quad_tree_system(
    mut commands: Commands,
    config: Res<QuadTreeConfig>,
    particles: Query<(&Position, &Mass)>,
    extent: Res<GlobalExtent>,
) {
    let particles: Vec<_> = particles
        .iter()
        .map(|(pos, mass)| (pos.0, **mass))
        .collect();
    let quadtree = QuadTree::new(&config, particles, &extent);
    commands.insert_resource(quadtree);
}

fn domain_decomposition_system(tree: Res<QuadTree>, config: Res<QuadTreeConfig>) {
    // Use the particle counts at depth config.min_depth for
    // decomposition for now. This obviously needs to be fixed and
    // replaced by a proper peano hilbert curve on an actual tree
}
