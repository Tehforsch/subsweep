use bevy::prelude::*;
use mpi::traits::Equivalence;

mod exchange_data_plugin;
mod extent;
// mod peano_hilbert;
pub mod quadtree;

pub use self::exchange_data_plugin::ExchangeDataPlugin;
use self::exchange_data_plugin::OutgoingEntities;
use self::extent::Extent;
use self::quadtree::QuadTree;
use self::quadtree::QuadTreeConfig;
use self::quadtree::QuadTreeIndex;
use crate::communication::AllGatherCommunicator;
use crate::communication::CollectiveCommunicator;
use crate::communication::CommunicationPlugin;
use crate::communication::CommunicationType;
use crate::communication::DataByRank;
use crate::communication::WorldRank;
use crate::communication::WorldSize;
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
        app.insert_resource(GlobalExtent(Extent::sentinel()))
            .insert_resource(TopLevelIndices::default());
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
            distribute_top_level_nodes_system,
        )
        .add_system_to_stage(
            DomainDecompositionStages::Decomposition,
            domain_decomposition_system.after(distribute_top_level_nodes_system),
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
    particles: Query<(Entity, &Position, &Mass)>,
    extent: Res<GlobalExtent>,
) {
    let particles: Vec<_> = particles
        .iter()
        .map(|(entity, pos, mass)| (entity, pos.0, **mass))
        .collect();
    let quadtree = QuadTree::new(&config, particles, &extent);
    commands.insert_resource(quadtree);
}

fn sum_vecs(mut data: DataByRank<Vec<usize>>) -> Vec<usize> {
    let mut sum = data.remove(&0).unwrap();
    for (_, other_result) in data.drain_all() {
        debug_assert_eq!(sum.len(), other_result.len());
        for i in 0..other_result.len() {
            sum[i] += other_result[i];
        }
    }
    sum
}

fn get_cutoffs(particle_counts: &[usize], num_ranks: usize) -> Vec<usize> {
    let total_work: usize = particle_counts.iter().sum();
    let work_per_rank = total_work / num_ranks;
    let mut key_cutoffs_by_rank = vec![0];
    let mut load = 0;
    for (i, count) in particle_counts.iter().enumerate() {
        load += count;
        if load >= work_per_rank {
            key_cutoffs_by_rank.push(i);
            if key_cutoffs_by_rank.len() == num_ranks {
                break;
            }
            load = 0;
        }
    }
    let num_entries_to_fill = num_ranks - key_cutoffs_by_rank.len();
    if num_entries_to_fill > 0 {
        panic!("One rank has no work");
    }
    // Even if num_entries_to_fill is zero, we add the final index once to make calculating the index
    // ranges later easier (since we can just use cutoffs[rank]..cutoffs[rank+1], even for the last rank)
    key_cutoffs_by_rank.extend((0..1 + num_entries_to_fill).map(|_| particle_counts.len()));
    key_cutoffs_by_rank
}

#[derive(Default, Deref, DerefMut)]
struct TopLevelIndices(DataByRank<Vec<QuadTreeIndex>>);

fn distribute_top_level_nodes_system(
    tree: Res<QuadTree>,
    config: Res<QuadTreeConfig>,
    num_ranks: Res<WorldSize>,
    mut indices: ResMut<TopLevelIndices>,
    mut comm: NonSendMut<AllGatherCommunicator<usize>>,
) {
    // Use the particle counts at depth config.min_depth for
    // decomposition for now. This obviously needs to be fixed and
    // replaced by a proper peano hilbert curve on an actual tree
    let top_level_tree_leaf_indices: Vec<_> =
        QuadTreeIndex::iter_all_nodes_at_depth(config.min_depth).collect();
    let buffer: Vec<_> = top_level_tree_leaf_indices
        .iter()
        .map(|index| tree[&index].data.num_particles())
        .collect();
    // replace with allreduce over buffer at some point
    let particles_per_leaf = sum_vecs(comm.all_gather_vec(&buffer));
    let cutoffs = get_cutoffs(&particles_per_leaf, **num_ranks);
    *indices = TopLevelIndices(
        (0..**num_ranks)
            .map(|rank| {
                let start = cutoffs[rank];
                let end = cutoffs[rank + 1];
                (
                    rank as i32,
                    top_level_tree_leaf_indices[start..end]
                        .iter()
                        .map(|x| *x)
                        .collect::<Vec<_>>(),
                )
            })
            .collect(),
    );
}

fn domain_decomposition_system(
    mut outgoing_entities: ResMut<OutgoingEntities>,
    tree: Res<QuadTree>,
    indices: Res<TopLevelIndices>,
    world_rank: Res<WorldRank>,
) {
    for (rank, indices) in indices.iter() {
        if *rank != **world_rank {
            for index in indices.iter() {
                tree[index].depth_first_map_leaf(&mut |_, leaf| {
                    for particle in leaf.iter() {
                        outgoing_entities.add(*rank, particle.entity);
                    }
                });
            }
        }
    }
}
