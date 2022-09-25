use bevy::prelude::*;
use mpi::traits::Equivalence;

mod exchange_data_plugin;
pub mod extent;
pub use self::exchange_data_plugin::ExchangeDataPlugin;
use self::exchange_data_plugin::OutgoingEntities;
use self::extent::Extent;
use crate::communication::CommunicatedOption;
use crate::communication::CommunicationPlugin;
use crate::communication::CommunicationType;
use crate::communication::Communicator;
use crate::communication::DataByRank;
use crate::communication::Rank;
use crate::communication::WorldRank;
use crate::communication::WorldSize;
use crate::mass::Mass;
use crate::named::Named;
use crate::physics::MassMoments;
use crate::position::Position;
use crate::quadtree::LeafData;
use crate::quadtree::QuadTree;
use crate::quadtree::QuadTreeConfig;
use crate::quadtree::QuadTreeIndex;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;
use crate::velocity::Velocity;

#[derive(StageLabel)]
pub enum DomainDecompositionStages {
    TopLevelTreeConstruction,
    Decomposition,
    Exchange,
}

#[derive(Named)]
pub struct DomainDecompositionPlugin;

impl RaxiomPlugin for DomainDecompositionPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.insert_resource(GlobalExtent(Extent::default()))
            .insert_resource(TopLevelIndices::default())
            .add_parameter_type::<QuadTreeConfig>()
            .insert_resource(QuadTree::make_empty_leaf_from_extent(Extent::default()))
            .add_system_to_stage(
                DomainDecompositionStages::TopLevelTreeConstruction,
                determine_global_extent_system,
            )
            .add_startup_system_to_stage(StartupStage::PostStartup, determine_global_extent_system)
            .add_system_to_stage(
                DomainDecompositionStages::TopLevelTreeConstruction,
                construct_quad_tree_system.after(determine_global_extent_system),
            )
            .add_system_to_stage(
                DomainDecompositionStages::TopLevelTreeConstruction,
                communicate_mass_moments_system.after(construct_quad_tree_system),
            )
            .add_system_to_stage(
                DomainDecompositionStages::Decomposition,
                distribute_top_level_nodes_system,
            )
            .add_system_to_stage(
                DomainDecompositionStages::Decomposition,
                domain_decomposition_system.after(distribute_top_level_nodes_system),
            )
            .add_plugin(CommunicationPlugin::<CommunicatedOption<Extent>>::new(
                CommunicationType::AllGather,
            ))
            .add_plugin(CommunicationPlugin::<MassMoments>::new(
                CommunicationType::AllGather,
            ));
    }
}

#[derive(Debug, Deref, DerefMut)]
pub struct GlobalExtent(Extent);

pub(super) fn determine_global_extent_system(
    particles: Query<&Position>,
    mut extent_communicator: Communicator<CommunicatedOption<Extent>>,
    mut global_extent: ResMut<GlobalExtent>,
) {
    let extent = Extent::from_positions(particles.iter().map(|x| &x.0));
    let all_extents = (*extent_communicator).all_gather(&extent.into());
    let all_extents: Vec<Extent> = all_extents.into_iter().filter_map(|x| x.into()).collect();
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

pub fn construct_quad_tree_system(
    config: Res<QuadTreeConfig>,
    particles: Query<(Entity, &Position, &Mass)>,
    extent: Res<GlobalExtent>,
    mut quadtree: ResMut<QuadTree>,
) {
    let particles: Vec<_> = particles
        .iter()
        .map(|(entity, pos, mass)| LeafData {
            entity,
            pos: pos.0,
            mass: **mass,
        })
        .collect();
    *quadtree = QuadTree::new(&config, particles, &extent);
}

fn sum_vecs(mut data: DataByRank<Vec<MassMoments>>) -> Vec<MassMoments> {
    let mut sum = data.remove(&0).unwrap();
    for (_, other_result) in data.drain_all() {
        debug_assert_eq!(sum.len(), other_result.len());
        for i in 0..other_result.len() {
            sum[i] += &other_result[i];
        }
    }
    sum
}

fn get_cutoffs(particle_counts: &[usize], num_ranks: usize) -> Vec<usize> {
    let total_work: usize = particle_counts.iter().sum();
    let mut work_per_rank = total_work / num_ranks;
    let mut key_cutoffs_by_rank = vec![0];
    let mut load = 0;
    let mut loads = vec![];
    let remaining_work = |loads: &[usize]| total_work - loads.iter().sum::<usize>();
    for (i, count) in particle_counts.iter().enumerate() {
        if load >= work_per_rank {
            key_cutoffs_by_rank.push(i);
            loads.push(load);
            // Recalculate work_per_rank based on the remaining work
            if key_cutoffs_by_rank.len() >= num_ranks {
                break;
            }
            work_per_rank = remaining_work(&loads) / (num_ranks - loads.len());
            load = 0;
        }
        load += count;
    }
    loads.push(remaining_work(&loads));
    let max_load = *loads.iter().max().unwrap() as f64;
    let min_load = *loads.iter().min().unwrap() as f64;
    let load_imbalance = (max_load - min_load) / max_load;
    debug!("Load imbalance: {:.1}%", (load_imbalance * 100.0));
    let num_entries_to_fill = num_ranks as i32 - key_cutoffs_by_rank.len() as i32;
    if num_entries_to_fill > 0 {
        error!("One rank has no work - increase domain min_depth");
    }
    // Even if num_entries_to_fill is zero, we add the final index once to make calculating the index
    // ranges later easier (since we can just use cutoffs[rank]..cutoffs[rank+1], even for the last rank)
    key_cutoffs_by_rank.extend((0..1 + num_entries_to_fill).map(|_| particle_counts.len()));
    if num_entries_to_fill > 0 {
        for (i, window) in key_cutoffs_by_rank.windows(2).enumerate() {
            println!(
                "{} {}",
                i,
                (window[0]..window[1])
                    .map(|x| particle_counts[x].to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
    }
    key_cutoffs_by_rank
}

#[derive(Default, Deref, DerefMut)]
pub struct TopLevelIndices(DataByRank<Vec<QuadTreeIndex>>);

impl TopLevelIndices {
    pub fn flat_iter(&self) -> impl Iterator<Item = (Rank, &QuadTreeIndex)> {
        self.0
            .iter()
            .flat_map(|(rank, indices)| indices.iter().map(|index| (*rank, index)))
    }
}

fn get_top_level_indices(depth: usize) -> Vec<QuadTreeIndex> {
    QuadTreeIndex::iter_all_nodes_at_depth(depth).collect()
}

pub fn communicate_mass_moments_system(
    mut tree: ResMut<QuadTree>,
    config: Res<QuadTreeConfig>,
    mut comm: Communicator<MassMoments>,
) {
    // Use the particle counts at depth config.min_depth for
    // decomposition for now. This obviously needs to be fixed and
    // replaced by a proper peano hilbert curve on an actual tree
    let top_level_tree_leaf_indices = get_top_level_indices(config.min_depth);
    let mass_moments: Vec<_> = top_level_tree_leaf_indices
        .iter()
        .map(|index| tree[index].data.moments.clone())
        .collect();
    // replace with allreduce over buffer at some point
    let total_mass_moments = sum_vecs(comm.all_gather_vec(&mass_moments));
    for (index, moments) in top_level_tree_leaf_indices
        .iter()
        .zip(total_mass_moments.iter())
    {
        tree[index].data.moments = moments.clone();
    }
}

fn distribute_top_level_nodes_system(
    tree: Res<QuadTree>,
    config: Res<QuadTreeConfig>,
    num_ranks: Res<WorldSize>,
    mut indices: ResMut<TopLevelIndices>,
) {
    let top_level_tree_leaf_indices = get_top_level_indices(config.min_depth);
    let particles_per_leaf: Vec<usize> = top_level_tree_leaf_indices
        .iter()
        .map(|index| tree[index].data.moments.count())
        .collect();
    let cutoffs = get_cutoffs(&particles_per_leaf, **num_ranks);
    *indices = TopLevelIndices(
        (0..**num_ranks)
            .map(|rank| {
                let start = cutoffs[rank];
                let end = cutoffs[rank + 1];
                (
                    rank as i32,
                    top_level_tree_leaf_indices[start..end].to_vec(),
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
