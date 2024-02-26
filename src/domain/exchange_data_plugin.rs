use std::marker::PhantomData;

use bevy_ecs::prelude::Commands;
use bevy_ecs::prelude::Component;
use bevy_ecs::prelude::Entity;
use bevy_ecs::prelude::IntoSystemDescriptor;
use bevy_ecs::prelude::Res;
use bevy_ecs::prelude::ResMut;
use bevy_ecs::prelude::Resource;
use bevy_ecs::system::AsSystemLabel;
use derive_more::Deref;
use derive_more::DerefMut;
use mpi::traits::Equivalence;
use mpi::traits::MatchesRaw;

use crate::communication::DataByRank;
use crate::communication::ExchangeCommunicator;
use crate::communication::Rank;
use crate::communication::SizedCommunicator;
use crate::communication::WorldRank;
use crate::communication::WorldSize;
use crate::named::Named;
use crate::prelude::LocalParticle;
use crate::prelude::Particles;
use crate::prelude::StartupStages;
use crate::simulation::Simulation;
use crate::simulation::SubsweepPlugin;

#[derive(Named)]
struct ExchangeDataStartupOrder;

#[derive(Default, Deref, DerefMut, Resource)]
pub(super) struct OutgoingEntities(DataByRank<Vec<Entity>>);

impl OutgoingEntities {
    pub fn add(&mut self, rank: Rank, entity: Entity) {
        self[rank].push(entity);
    }
}

#[derive(Default, Deref, DerefMut, Resource)]
struct SpawnedEntities(DataByRank<Vec<Entity>>);

#[derive(Deref, DerefMut, Resource)]
struct ExchangeBuffers<T>(DataByRank<Vec<T>>);

impl<T> ExchangeBuffers<T> {
    fn take(&mut self) -> DataByRank<Vec<T>> {
        std::mem::take(&mut self.0)
    }
}

#[derive(Named)]
pub struct ExchangeDataPlugin<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for ExchangeDataPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

#[derive(Equivalence, Deref, DerefMut)]
struct NumEntities(usize);

impl<T: Sync + Send + 'static + Component + Clone + Equivalence> SubsweepPlugin
    for ExchangeDataPlugin<T>
where
    <T as Equivalence>::Out: MatchesRaw,
{
    fn allow_adding_twice(&self) -> bool {
        true
    }

    fn build_once_everywhere(&self, sim: &mut Simulation) {
        let rank = **sim.unwrap_resource::<WorldRank>();
        let size = **sim.unwrap_resource::<WorldSize>();
        sim.insert_resource(OutgoingEntities(DataByRank::from_size_and_rank(size, rank)))
            .insert_resource(SpawnedEntities(DataByRank::from_size_and_rank(size, rank)));
        sim.add_startup_system_to_stage(StartupStages::Exchange, despawn_outgoing_entities_system)
            .add_startup_system_to_stage(
                StartupStages::Exchange,
                reset_outgoing_entities_system.after(despawn_outgoing_entities_system),
            )
            .add_startup_system_to_stage(StartupStages::Exchange, spawn_incoming_entities_system);
    }

    fn build_everywhere(&self, sim: &mut Simulation) {
        let rank = **sim.unwrap_resource::<WorldRank>();
        let size = **sim.unwrap_resource::<WorldSize>();
        sim.insert_resource(ExchangeBuffers::<T>(DataByRank::from_size_and_rank(
            size, rank,
        )));
        sim.add_well_ordered_system_to_startup_stage::<_, ExchangeDataStartupOrder>(
            StartupStages::Exchange,
            Self::exchange_buffers_system
                .after(Self::fill_buffers_system)
                .after(spawn_incoming_entities_system)
                .before(reset_outgoing_entities_system),
            Self::exchange_buffers_system.as_system_label(),
        )
        .add_startup_system_to_stage(StartupStages::Exchange, Self::fill_buffers_system)
        .add_startup_system_to_stage(
            StartupStages::Exchange,
            Self::reset_buffers_system.after(Self::exchange_buffers_system),
        );
    }
}

impl<T: Sync + Send + 'static + Component + Clone + Equivalence> ExchangeDataPlugin<T> {
    fn fill_buffers_system(
        entity_exchange: Res<OutgoingEntities>,
        query: Particles<&T>,
        mut buffer: ResMut<ExchangeBuffers<T>>,
    ) {
        for (rank, entities) in entity_exchange.iter() {
            // This allocates a new buffer every time. An alternative would be
            // to keep this at maximum size, trading performance for memory overhead
            buffer.insert(
                rank,
                entities
                    .iter()
                    .map(|entity| query.get(*entity).unwrap().clone())
                    .collect(),
            );
        }
    }

    fn exchange_buffers_system(
        mut commands: Commands,
        mut buffers: ResMut<ExchangeBuffers<T>>,
        spawned_entities: Res<SpawnedEntities>,
    ) {
        let mut communicator = ExchangeCommunicator::<T>::new();
        let buffers = buffers.take();
        let mut incoming = communicator.exchange_all(buffers);
        for (rank, data) in incoming.drain_all() {
            let spawned_entities = spawned_entities[rank].clone();
            for (entity, component) in spawned_entities.iter().zip(data.into_iter()) {
                commands.entity(*entity).insert(component);
            }
        }
    }

    fn reset_buffers_system(
        mut buffers: ResMut<ExchangeBuffers<T>>,
        size: Res<WorldSize>,
        rank: Res<WorldRank>,
    ) {
        *buffers = ExchangeBuffers(DataByRank::from_size_and_rank(**size, **rank));
    }
}

fn spawn_incoming_entities_system(
    mut commands: Commands,
    mut spawned_entities: ResMut<SpawnedEntities>,
    num_outgoing: Res<OutgoingEntities>,
) {
    let mut communicator: ExchangeCommunicator<NumEntities> = ExchangeCommunicator::new();
    let data: DataByRank<Vec<NumEntities>> = communicator
        .other_ranks()
        .into_iter()
        .map(|rank| {
            (
                rank,
                vec![NumEntities(num_outgoing.get(&rank).unwrap().len())],
            )
        })
        .collect();
    let incoming = communicator.exchange_all(data);
    for (rank, num_incoming) in incoming {
        let num_incoming = &num_incoming[0];
        spawned_entities.insert(
            rank,
            (0..**num_incoming)
                .map(|_| commands.spawn(LocalParticle).id())
                .collect(),
        );
    }
}

fn reset_outgoing_entities_system(
    mut outgoing: ResMut<OutgoingEntities>,
    size: Res<WorldSize>,
    rank: Res<WorldRank>,
) {
    *outgoing = OutgoingEntities(DataByRank::from_size_and_rank(**size, **rank));
}

fn despawn_outgoing_entities_system(
    mut commands: Commands,
    entity_exchange: Res<OutgoingEntities>,
) {
    for (_, entities) in entity_exchange.iter() {
        for entity in entities {
            commands.entity(*entity).despawn();
        }
    }
}
