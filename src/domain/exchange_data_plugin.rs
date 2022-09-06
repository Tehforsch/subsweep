use std::marker::PhantomData;

use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Entity;
use bevy::prelude::NonSendMut;
use bevy::prelude::ParallelSystemDescriptorCoercion;
use bevy::prelude::Plugin;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::SystemSet;
use mpi::traits::Equivalence;

use super::DomainDecompositionStages;
use crate::communication::DataByRank;
use crate::communication::ExchangeCommunicator;
use crate::communication::Rank;
use crate::communication::SizedCommunicator;

struct ExchangePluginExists;

struct OutgoingEntities(DataByRank<Vec<Entity>>);

struct SpawnedEntities(DataByRank<Vec<Entity>>);

struct NumOutgoingEntities(DataByRank<usize>);

struct ExchangeBuffers<T>(DataByRank<Vec<T>>);

struct ExchangeDataPlugin<T> {
    _marker: PhantomData<T>,
}

#[derive(Equivalence)]
struct NumEntities(usize);

impl<T> Default for ExchangeDataPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData::default(),
        }
    }
}

impl<T: Sync + Send + 'static + Component + Clone + Equivalence> Plugin for ExchangeDataPlugin<T> {
    fn build(&self, app: &mut bevy::prelude::App) {
        let exists = app.world.get_resource_mut::<ExchangePluginExists>();
        let first = exists.is_none();
        if first {
            app.world.insert_resource(ExchangePluginExists);
        }
        app.add_system_set_to_stage(
            DomainDecompositionStages::Exchange,
            SystemSet::new()
                .with_system(Self::fill_buffers_system)
                .with_system(Self::despawn_outgoing_entities_system)
                .with_system(Self::send_buffers_system.after(Self::fill_buffers_system))
                .with_system(Self::receive_buffers_system.after(Self::send_buffers_system)),
        );
        if first {
            app.add_system_to_stage(
                DomainDecompositionStages::Exchange,
                send_num_outgoing_entities_system.after(Self::receive_buffers_system),
            )
            .add_system_to_stage(
                DomainDecompositionStages::Exchange,
                spawn_incoming_entities_system.after(send_num_outgoing_entities_system),
            );
        }
    }
}

impl<T: Sync + Send + 'static + Component + Clone + Equivalence> ExchangeDataPlugin<T> {
    fn fill_buffers_system(
        entity_exchange: Res<OutgoingEntities>,
        query: Query<&T>,
        mut buffer: ResMut<ExchangeBuffers<T>>,
    ) {
        for (rank, entities) in entity_exchange.0.iter() {
            let num_exchanged = entities.len();
            // This allocates a new buffer every time. An alternative would be
            // to keep this at maximum size, trading performance for memory overhead
            buffer.0.insert(*rank, Vec::with_capacity(num_exchanged));
            for (i, entity) in entities.iter().enumerate() {
                buffer.0[*rank as Rank][i] = query.get(*entity).unwrap().clone();
            }
        }
    }

    fn despawn_outgoing_entities_system(
        mut commands: Commands,
        entity_exchange: Res<OutgoingEntities>,
    ) {
        for (_, entities) in entity_exchange.0.iter() {
            for entity in entities {
                commands.entity(*entity).despawn();
            }
        }
    }

    fn send_buffers_system(
        mut communicator: NonSendMut<ExchangeCommunicator<T>>,
        mut buffers: ResMut<ExchangeBuffers<T>>,
    ) {
        for (rank, data) in buffers.0.drain_all() {
            communicator.send_vec(rank, data);
        }
    }

    fn receive_buffers_system(
        mut commands: Commands,
        mut communicator: NonSendMut<ExchangeCommunicator<T>>,
        spawned_entities: Res<SpawnedEntities>,
    ) {
        for (rank, data) in communicator.receive_vec() {
            let spawned_entities = spawned_entities.0[rank].clone();
            let iterator = spawned_entities
                .into_iter()
                .zip(data.into_iter().map(|component| (component,)));
            commands.insert_or_spawn_batch(iterator);
        }
    }
}

fn send_num_outgoing_entities_system(
    mut communicator: NonSendMut<ExchangeCommunicator<NumEntities>>,
    num_outgoing: Res<NumOutgoingEntities>,
) {
    for rank in communicator.other_ranks() {
        communicator.send(rank, NumEntities(num_outgoing.0[rank]));
    }
}

fn spawn_incoming_entities_system(
    mut commands: Commands,
    mut communicator: NonSendMut<ExchangeCommunicator<NumEntities>>,
    mut spawned_entities: ResMut<SpawnedEntities>,
) {
    for (rank, num_incoming) in communicator.receive() {
        spawned_entities.0[rank] = (0..num_incoming.0)
            .map(|_| {
                let id = commands.spawn().id();
                id
            })
            .collect();
    }
}
