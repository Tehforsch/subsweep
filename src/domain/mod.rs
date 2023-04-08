use bevy::prelude::*;
use bimap::BiMap;
use derive_custom::raxiom_parameters;

pub mod decomposition;
mod exchange_data_plugin;
pub mod extent;
mod key;
mod quadtree;
mod work;

use self::decomposition::KeyCounter;
use self::decomposition::ParallelCounter;
pub use self::exchange_data_plugin::ExchangeDataPlugin;
use self::exchange_data_plugin::OutgoingEntities;
pub use self::extent::Extent;
pub use self::quadtree::LeafData;
pub use self::quadtree::NodeData;
pub use self::quadtree::QuadTree;
use self::work::Work;
use crate::communication::CommunicatedOption;
use crate::communication::CommunicationPlugin;
use crate::communication::WorldRank;
use crate::components::Position;
use crate::named::Named;
use crate::peano_hilbert::PeanoKey3d;
use crate::prelude::Communicator;
use crate::prelude::ParticleId;
use crate::prelude::Particles;
use crate::prelude::SimulationStartupStages;
use crate::prelude::WorldSize;
use crate::quadtree::QuadTreeConfig;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;

#[cfg(feature = "2d")]
pub type DomainKey = PeanoKey2d;
#[cfg(feature = "3d")]
pub type DomainKey = PeanoKey3d;
pub type Decomposition = decomposition::Decomposition<DomainKey>;

/// Parameters of the domain tree. See [QuadTreeConfig](crate::quadtree::QuadTreeConfig)
#[raxiom_parameters("tree")]
pub struct TreeParameters {
    #[serde(default)]
    pub tree: QuadTreeConfig,
}

impl Default for TreeParameters {
    fn default() -> Self {
        Self {
            tree: default_domain_tree_params(),
        }
    }
}

fn default_domain_tree_params() -> QuadTreeConfig {
    QuadTreeConfig {
        ..Default::default()
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct IdEntityMap(BiMap<ParticleId, Entity>);

#[derive(StageLabel)]
pub enum DomainStartupStages {
    DetermineGlobalExtents,
    Decomposition,
    SetOutgoingEntities,
    Exchange,
    TreeConstruction,
}

#[derive(StageLabel)]
pub enum DomainStages {
    TopLevelTreeConstruction,
    Decomposition,
    Exchange,
}

#[derive(Named)]
pub struct DomainPlugin;

impl RaxiomPlugin for DomainPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.insert_resource(GlobalExtent(Extent::default()))
            .add_plugin(CommunicationPlugin::<CommunicatedOption<Extent>>::default())
            .add_plugin(CommunicationPlugin::<Work>::default())
            .add_parameter_type::<TreeParameters>();
        sim.add_startup_system_to_stage(
            SimulationStartupStages::InsertDerivedComponents,
            determine_particle_ids_system,
        )
        .add_startup_system_to_stage(
            DomainStartupStages::DetermineGlobalExtents,
            determine_global_extent_system,
        )
        .add_startup_system_to_stage(
            DomainStartupStages::Decomposition,
            domain_decomposition_system,
        )
        .add_startup_system_to_stage(
            DomainStartupStages::SetOutgoingEntities,
            set_outgoing_entities_system,
        )
        .add_startup_system_to_stage(
            DomainStartupStages::TreeConstruction,
            update_id_entity_map_system,
        )
        .add_startup_system_to_stage(
            DomainStartupStages::TreeConstruction,
            construct_quad_tree_system,
        );
    }
}

#[derive(Debug, Deref, DerefMut, Resource)]
pub struct GlobalExtent(Extent);

pub fn construct_quad_tree_system(
    mut commands: Commands,
    config: Res<TreeParameters>,
    particles: Particles<(&ParticleId, &Position)>,
    extent: Res<GlobalExtent>,
) {
    debug!("Constructing top level tree");
    let particles: Vec<_> = particles
        .iter()
        .map(|(id, pos)| LeafData {
            id: *id,
            pos: pos.0,
        })
        .collect();
    commands.insert_resource(QuadTree::new(&config.tree, particles, &extent));
}

pub(super) fn determine_global_extent_system(
    particles: Particles<&Position>,
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

fn determine_particle_ids_system(
    mut commands: Commands,
    world_rank: Res<WorldRank>,
    particles: Particles<Entity>,
) {
    // Ugly and hacky but most likely safe and nice for debugging.
    const MAX_NUM_PARTICLES_PER_RANK: u64 = 1000000000;
    if particles.iter().count() as u64 > MAX_NUM_PARTICLES_PER_RANK {
        panic!("Too many particles on rank - change ID scheme to account for this");
    }
    let mut map = BiMap::default();
    for (i, entity) in particles.iter().enumerate() {
        let id: u64 = MAX_NUM_PARTICLES_PER_RANK * (**world_rank as u64) + i as u64;
        let id = ParticleId(id);
        commands.entity(entity).insert(id);
        map.insert(id, entity);
    }
    commands.insert_resource(IdEntityMap(map))
}

fn update_id_entity_map_system(query: Query<(&ParticleId, Entity)>, mut map: ResMut<IdEntityMap>) {
    map.0 = query.iter().map(|(id, entity)| (*id, entity)).collect();
}

fn domain_decomposition_system(
    mut commands: Commands,
    global_extent: Res<GlobalExtent>,
    particles: Particles<&Position>,
    world_size: Res<WorldSize>,
    mut comm: Communicator<Work>,
) {
    let local_keys = particles
        .iter()
        .map(|p| PeanoKey3d::from_point_and_extent(**p, &global_extent.0))
        .collect();
    let local_counter = KeyCounter::new(local_keys);
    let mut counter = ParallelCounter {
        local_counter,
        comm: &mut *comm,
    };
    let decomp = Decomposition::new(&mut counter, **world_size);
    decomp.log_imbalance();
    commands.insert_resource(decomp);
}

fn set_outgoing_entities_system(
    mut outgoing_entities: ResMut<OutgoingEntities>,
    decomposition: Res<Decomposition>,
    world_rank: Res<WorldRank>,
    global_extent: Res<GlobalExtent>,
    particles: Particles<(Entity, &Position)>,
) {
    for (entity, pos) in particles.iter() {
        let key = PeanoKey3d::from_point_and_extent(**pos, &global_extent.0);
        let rank = decomposition.get_owning_rank(key);
        if rank != **world_rank {
            outgoing_entities.add(rank, entity);
        }
    }
}
